# Run Rosenpass on Hermit

## Required patches

### liboqs 0.9

https://github.com/open-quantum-safe/liboqs/pull/2104

```diff
diff --git a/src/common/CMakeLists.txt b/src/common/CMakeLists.txt
index c077b489..1364096f 100644
--- a/src/common/CMakeLists.txt
+++ b/src/common/CMakeLists.txt
@@ -84,8 +84,9 @@ add_library(common OBJECT ${AES_IMPL}
 if(${OQS_USE_OPENSSL})
     target_include_directories(common PRIVATE ${OPENSSL_INCLUDE_DIR})
 else()
-    check_symbol_exists(getentropy "unistd.h;sys/random.h" CMAKE_HAVE_GETENTROPY)
-    if(${CMAKE_HAVE_GETENTROPY})
+    check_symbol_exists(getentropy "unistd.h" CMAKE_UNISTD_HAVE_GETENTROPY)
+    check_symbol_exists(getentropy "sys/random.h" CMAKE_SYS_RANDOM_HAVE_GETENTROPY)
+    if("${CMAKE_UNISTD_HAVE_GETENTROPY}" OR "${CMAKE_SYS_RANDOM_HAVE_GETENTROPY}")
         target_compile_definitions(common PRIVATE OQS_HAVE_GETENTROPY)
     endif()
 endif()
```

### Hermit Kernel

Increase stack size:

```diff
diff --git a/src/config.rs b/src/config.rs
index 6f9581a06..85393e314 100644
--- a/src/config.rs
+++ b/src/config.rs
@@ -2,7 +2,7 @@ pub(crate) const KERNEL_STACK_SIZE: usize = 0x8000;
 
 pub const DEFAULT_STACK_SIZE: usize = 0x0001_0000;
 
-pub(crate) const USER_STACK_SIZE: usize = 0x0010_0000;
+pub(crate) const USER_STACK_SIZE: usize = 0x0100_0000;
 
 #[cfg(any(
        all(any(feature = "tcp", feature = "udp"), not(feature = "rtl8139")),
```

## Run Rosenpass

Terminal 1 (Hermit):

```bash
# move to the directory containing the rosenpass and hermit-rs repos
docker run --rm -it -v .:/mnt -w /mnt --privileged --name hermit-rosenpass ghcr.io/hermit-os/hermit-gcc:latest
cd rosenpass
./run.sh
```

Terminal 2 (Linux):

```bash
docker exec -it hermit-rosenpass bash
cargo run --bin rosenpass --no-default-features -- exchange-config debian-rosenpass-config.toml
```
