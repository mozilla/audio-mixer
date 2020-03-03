use crate::{Channel, Mixer};
use std::os::raw::c_void;
use std::slice;

// C APIs
// ------------------------------------------------------------------------------------------------
#[no_mangle]
pub extern "C" fn mixer_create(
    format: Format,
    input_channels: Channels,
    output_channels: Channels,
) -> Handle {
    // Give up the onwership of the memory owned by the Box and return that memory's address.
    TypedMixer::into_handle(Box::new(TypedMixer::new(
        &format,
        input_channels.to_slice(),
        output_channels.to_slice(),
    )))
}

#[no_mangle]
pub extern "C" fn mixer_destroy(handle: Handle) {
    // Take the onwership of the memory pointed by the handle.
    let _mixer = unsafe { TypedMixer::from_handle(handle) };
}

#[no_mangle]
pub extern "C" fn mixer_mix(
    handle: Handle,
    input_buffer: *const c_void,
    input_buffer_size: usize,
    output_buffer: *mut c_void,
    output_buffer_size: usize,
) {
    // Get the mixer from handle, without taking the onwership of the memory pointed by the handle.
    let mixer = unsafe { TypedMixer::borrow_from_handle(handle) };
    mixer.expect("Handle is empty!").mix(
        input_buffer,
        input_buffer_size,
        output_buffer,
        output_buffer_size,
    );
}

// Exposed data type
// ------------------------------------------------------------------------------------------------
#[allow(dead_code)] // To avoid clippy warnings.
#[repr(C)]
pub enum Format {
    F32,
    I16,
}

#[repr(C)]
pub struct Channels {
    begin: *const Channel,
    length: usize,
}

impl Channels {
    fn to_slice(&self) -> &[Channel] {
        assert_ability_to_form_valid_slice(self.begin, self.length);
        unsafe { slice::from_raw_parts(self.begin, self.length) }
    }
}

// Use an opaque pointer as handle. It should be converted to a similar form like:
// ```
// struct TypedMixer;
// typedef struct TypedMixer* Handle;
// ```
// when it's exposed to C/C++.
type Handle = *mut TypedMixer;

// No `#[repr(C)]` here so this should be converted to an opaque data struct: `struct TypedMixer`.
pub enum TypedMixer {
    FloatMixer(Mixer<f32>),
    IntegerMixer(Mixer<i16>),
}

impl TypedMixer {
    fn into_handle(mixer: Box<Self>) -> Handle {
        Box::into_raw(mixer)
    }

    unsafe fn from_handle(handle: Handle) -> Box<Self> {
        Box::from_raw(handle)
    }

    unsafe fn borrow_from_handle<'a>(handle: Handle) -> Option<&'a Self> {
        handle.as_ref()
    }

    fn new(format: &Format, input_channels: &[Channel], output_channels: &[Channel]) -> Self {
        match format {
            Format::F32 => Self::FloatMixer(Mixer::<f32>::new(input_channels, output_channels)),
            Format::I16 => Self::IntegerMixer(Mixer::<i16>::new(input_channels, output_channels)),
        }
    }

    fn mix(
        &self,
        input_buffer_ptr: *const c_void,
        input_buffer_size: usize,
        output_buffer_ptr: *mut c_void,
        output_buffer_size: usize,
    ) {
        match self {
            Self::FloatMixer(mixer) => {
                let (input_buffer, output_buffer) = Self::convert_to_buffers::<f32>(
                    input_buffer_ptr,
                    input_buffer_size,
                    output_buffer_ptr,
                    output_buffer_size,
                );
                mixer.mix(input_buffer, output_buffer);
            }
            Self::IntegerMixer(mixer) => {
                let (input_buffer, output_buffer) = Self::convert_to_buffers::<i16>(
                    input_buffer_ptr,
                    input_buffer_size,
                    output_buffer_ptr,
                    output_buffer_size,
                );
                mixer.mix(input_buffer, output_buffer);
            }
        }
    }

    fn convert_to_buffers<'a, T>(
        input_buffer_ptr: *const c_void,
        input_buffer_size: usize,
        output_buffer_ptr: *mut c_void,
        output_buffer_size: usize,
    ) -> (&'a [T], &'a mut [T]) {
        let input_buffer_ptr = input_buffer_ptr as *const T;
        let output_buffer_ptr = output_buffer_ptr as *mut T;

        assert_ability_to_form_valid_slice(input_buffer_ptr, input_buffer_size);
        assert_ability_to_form_valid_slice(output_buffer_ptr, output_buffer_size);

        unsafe {
            (
                slice::from_raw_parts(input_buffer_ptr, input_buffer_size),
                slice::from_raw_parts_mut(output_buffer_ptr, output_buffer_size),
            )
        }
    }
}

// Utilities
// ------------------------------------------------------------------------------------------------
// The slices used here cannot be empty. slice::from_raw_parts(_mut) doesn't check `size`
// so we do it by ourselves.
fn assert_ability_to_form_valid_slice<T>(ptr: *const T, size: usize) {
    assert!(!ptr.is_null());
    assert_ne!(size, 0);
}

// Tests
// ------------------------------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use super::*;
    use std::any::{Any, TypeId};
    use std::fmt::Debug;

    #[test]
    fn test_build_ffi() {
        use std::process::Command;

        let output = Command::new("make")
            .arg("-C")
            .arg("examples")
            .arg("clean")
            .arg("check")
            .output()
            .expect("failed to compile test files");

        println!("status: {}", output.status);
        println!("--- stdout ---");
        println!("{}", String::from_utf8_lossy(&output.stdout));
        println!("-- stderr ---");
        println!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.status.success());

        assert!(Command::new("make")
            .arg("-C")
            .arg("examples")
            .arg("clean")
            .output()
            .expect("failed to clean up compiled files")
            .status
            .success());
    }

    #[test]
    fn test_c_api() {
        test_c_api_by_type::<f32>();
        test_c_api_by_type::<i16>();
    }

    fn test_c_api_by_type<T: Any + Clone + Debug + Default + From<u8> + PartialEq>() {
        let (input_channels, input_buffer, output_channels, mut output_buffer) =
            get_test_data::<T>();

        let handle = mixer_create(
            get_format::<T>(),
            get_channels(&input_channels),
            get_channels(&output_channels),
        );
        mixer_mix(
            handle,
            input_buffer.as_ptr() as *const c_void,
            input_buffer.len(),
            output_buffer.as_mut_ptr() as *mut c_void,
            output_buffer.len(),
        );
        mixer_destroy(handle);

        println!("{:?} is mixed to {:?}", input_buffer, output_buffer);
        let expected = get_mixer_output::<T>(&input_channels, &input_buffer, &output_channels);
        assert_eq!(output_buffer, expected);

        fn get_format<T: Any>() -> Format {
            let type_id = TypeId::of::<T>();
            if type_id == TypeId::of::<f32>() {
                Format::F32
            } else if type_id == TypeId::of::<i16>() {
                Format::I16
            } else {
                panic!("Unsupported type!");
            }
        }

        fn get_channels(channels: &[Channel]) -> Channels {
            Channels {
                begin: channels.as_ptr(),
                length: channels.len(),
            }
        }
    }

    fn get_mixer_output<T: Any + Clone + Default>(
        input_channels: &[Channel],
        input_buffer: &[T],
        output_channels: &[Channel],
    ) -> Vec<T> {
        assert_eq!(input_channels.len(), input_buffer.len());
        let mut output_buffer = default_buffer::<T>(output_channels.len());

        if TypeId::of::<T>() == TypeId::of::<f32>() {
            let mixer = Mixer::<f32>::new(input_channels, output_channels);
            unsafe {
                mixer.mix(
                    &*(input_buffer as *const [T] as *const [f32]),
                    &mut *(output_buffer.as_mut_slice() as *mut [T] as *mut [f32]),
                );
            }
        } else if TypeId::of::<T>() == TypeId::of::<i16>() {
            let mixer = Mixer::<i16>::new(input_channels, output_channels);
            unsafe {
                mixer.mix(
                    &*(input_buffer as *const [T] as *const [i16]),
                    &mut *(output_buffer.as_mut_slice() as *mut [T] as *mut [i16]),
                );
            }
        } else {
            panic!("Unsupport type");
        }

        output_buffer
    }

    fn get_test_data<T: Clone + Default + From<u8>>() -> (Vec<Channel>, Vec<T>, Vec<Channel>, Vec<T>)
    {
        let input_channels = vec![
            Channel::FrontLeft,
            Channel::Silence,
            Channel::FrontRight,
            Channel::FrontCenter,
            Channel::BackLeft,
            Channel::SideRight,
            Channel::LowFrequency,
            Channel::SideLeft,
            Channel::BackCenter,
            Channel::BackRight,
        ];
        let output_channels = vec![Channel::Silence, Channel::FrontRight, Channel::FrontLeft];
        let (input_buffer, output_buffer) =
            create_buffers::<T>(input_channels.len(), output_channels.len());
        (input_channels, input_buffer, output_channels, output_buffer)
    }

    fn create_buffers<T: Clone + Default + From<u8>>(
        input_size: usize,
        output_size: usize,
    ) -> (Vec<T>, Vec<T>) {
        let mut input_buffer = default_buffer::<T>(input_size);
        for (i, data) in input_buffer.iter_mut().enumerate() {
            *data = T::from((i + 1) as u8);
        }
        let output_buffer = default_buffer::<T>(output_size);
        (input_buffer, output_buffer)
    }

    fn default_buffer<T: Clone + Default>(size: usize) -> Vec<T> {
        let mut v = Vec::with_capacity(size);
        v.resize(size, T::default());
        v
    }
}
