cmake_minimum_required(VERSION 3.22.1)
project(mirrorx_capture_jni LANGUAGES CXX)

add_library(mirrorx_media SHARED IMPORTED)
set_target_properties(mirrorx_media PROPERTIES
  IMPORTED_LOCATION "${CMAKE_CURRENT_LIST_DIR}/../jniLibs/${ANDROID_ABI}/libmirrorx_media.so"
)

add_library(mirrorx_capture_jni SHARED mirrorx_jni.cpp)
target_compile_features(mirrorx_capture_jni PRIVATE cxx_std_17)
target_include_directories(mirrorx_capture_jni PRIVATE
  "${CMAKE_CURRENT_LIST_DIR}/../../../../../../../crates/mirrorx-media/include"
)
target_link_libraries(mirrorx_capture_jni PRIVATE mirrorx_media android log)
