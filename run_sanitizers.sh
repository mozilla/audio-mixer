echo "Sanitizer Tests (core)\n========================================"
cd core
sh run_sanitizers.sh
cd ..

echo "Sanitizer Tests (capi)\n========================================"
cd capi
sh run_sanitizers.sh
cd ..
