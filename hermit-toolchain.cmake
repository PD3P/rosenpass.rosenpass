set(CMAKE_SYSTEM_NAME Hermit)
set(CMAKE_SYSTEM_PROCESSOR x86_64)

set(CMAKE_C_COMPILER x86_64-hermit-gcc)
set(CMAKE_CXX_COMPILER x86_64-hermit-g++)

set(CMAKE_EXE_LINKER_FLAGS_INIT "-Wl,--whole-archive /mnt/rosenpass/libhermit.a -Wl,--no-whole-archive")
