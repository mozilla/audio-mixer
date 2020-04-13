extern crate audio_mixer;
use audio_mixer::{Channel, Mixer};
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
        use std::process::{Command, Output};

        const C_FILE: &str = "examples/audio_mixer.c";
        const HEADER_DIR: &str = "include/";
        const LIBRARY_DIR: &str = "../target/debug/";
        const LIBRARY_NAME: &str = "audio_mixer_capi";
        const EXECUTABLE: &str = "examples/audio_mixer";

        build_ffi_library();

        let mut flags = vec![
            format!("-I{}", HEADER_DIR),
            "-L".to_string(),
            LIBRARY_DIR.to_string(),
            format!("-l{}", LIBRARY_NAME),
        ];

        // Some symbols used in our dependencies won't be linked automatically on Linux or Windows.
        if cfg!(target_os = "linux") {
            let mut dependencies = vec![
                "-lpthread".to_string(),
                "-Wl,--no-as-needed".to_string(),
                "-ldl".to_string(),
            ];
            flags.append(&mut dependencies);
        } else if cfg!(target_os = "windows") {
            let mut dependencies = vec!["-lWS2_32".to_string(), "-luserenv".to_string()];
            flags.append(&mut dependencies);
        }

        let build_executable = Command::new("c++")
            .arg(C_FILE)
            .args(&flags)
            .arg("-o")
            .arg(EXECUTABLE)
            .output()
            .expect("failed to build executable");
        print_command_message(&build_executable);
        assert!(build_executable.status.success());

        let run_executable = Command::new(EXECUTABLE)
            .output()
            .expect("failed to run executable");
        let output = String::from_utf8(run_executable.stdout).unwrap();
        println!("{}", output);
        assert!(run_executable.status.success());

        let remove_executable = Command::new("rm")
            .arg(EXECUTABLE)
            .output()
            .expect("failed to remove executable");
        print_command_message(&remove_executable);
        assert!(remove_executable.status.success());

        fn print_command_message(output: &Output) {
            let message = String::from_utf8(if output.status.success() {
                output.stdout.clone()
            } else {
                output.stderr.clone()
            })
            .unwrap();
            if !message.is_empty() {
                println!("{}", message);
            }
        }

        fn build_ffi_library() {
            use std::path::Path;

            const HEADER_NAME: &str = "audio_mixer.h";
            let lib_name = format!("lib{}", LIBRARY_NAME);
            let path = Path::new(LIBRARY_DIR)
                .join(lib_name.as_str())
                .with_extension("a");
            if path.exists() {
                assert!(
                    Path::new(HEADER_DIR).join(HEADER_NAME).exists(),
                    "No header gnerated but library exists!"
                );
                return;
            }

            let build_library = Command::new("cargo")
                .arg("build")
                .arg("--lib")
                .output()
                .expect("failed to build library");
            print_command_message(&build_library);
            assert!(build_library.status.success());
        }
    }

    #[test]
    fn test_c_api() {
        test_c_api_by_type::<f32>();
        test_c_api_by_type::<i16>();
    }

    fn test_c_api_by_type<T: Any + Clone + Debug + Default + From<u8>>() {
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
