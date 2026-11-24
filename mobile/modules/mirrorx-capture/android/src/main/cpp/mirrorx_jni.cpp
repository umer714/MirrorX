#include <jni.h>
#include <cstdint>
#include <vector>
#include "mirrorx_media.h"

namespace {
constexpr jlong kInvalidHandle = 0;
constexpr jint kMaxFrameBytes = 16 * 1024 * 1024;

MirrorXH264QueueHandle* from_handle(jlong handle) {
  return reinterpret_cast<MirrorXH264QueueHandle*>(static_cast<intptr_t>(handle));
}
}

extern "C" JNIEXPORT jlong JNICALL
Java_expo_modules_mirrorxcapture_MirrorXNativeQueue_nativeCreate(
    JNIEnv*, jclass, jint capacity) {
  if (capacity < 1 || capacity > 120) return kInvalidHandle;
  auto* queue = mirrorx_h264_queue_new(static_cast<size_t>(capacity));
  return static_cast<jlong>(reinterpret_cast<intptr_t>(queue));
}

extern "C" JNIEXPORT jint JNICALL
Java_expo_modules_mirrorxcapture_MirrorXNativeQueue_nativePush(
    JNIEnv* env, jclass, jlong handle, jbyteArray data, jboolean keyframe, jlong pts_micros) {
  if (handle == kInvalidHandle || data == nullptr) return -1;
  const jsize length = env->GetArrayLength(data);
  if (length <= 0 || length > kMaxFrameBytes) return -1;
  std::vector<uint8_t> bytes(static_cast<size_t>(length));
  env->GetByteArrayRegion(data, 0, length, reinterpret_cast<jbyte*>(bytes.data()));
  if (env->ExceptionCheck()) return -1;
  return mirrorx_h264_queue_push(from_handle(handle), bytes.data(), bytes.size(),
                                 keyframe == JNI_TRUE, static_cast<uint64_t>(pts_micros));
}

extern "C" JNIEXPORT jobject JNICALL
Java_expo_modules_mirrorxcapture_MirrorXNativeQueue_nativePop(
    JNIEnv* env, jclass, jlong handle, jint max_bytes) {
  if (handle == kInvalidHandle || max_bytes <= 0 || max_bytes > kMaxFrameBytes) return nullptr;
  std::vector<uint8_t> bytes(static_cast<size_t>(max_bytes));
  size_t length = 0;
  bool keyframe = false;
  uint64_t pts_micros = 0;
  const int32_t status = mirrorx_h264_queue_pop(from_handle(handle), bytes.data(), bytes.size(),
                                                 &length, &keyframe, &pts_micros);
  if (status != 1 || length > bytes.size()) return nullptr;
  jbyteArray payload = env->NewByteArray(static_cast<jsize>(length));
  if (payload == nullptr) return nullptr;
  env->SetByteArrayRegion(payload, 0, static_cast<jsize>(length),
                          reinterpret_cast<const jbyte*>(bytes.data()));
  if (env->ExceptionCheck()) return nullptr;

  jclass frame_class = env->FindClass("expo/modules/mirrorxcapture/MirrorXEncodedFrame");
  if (frame_class == nullptr) return nullptr;
  jmethodID constructor = env->GetMethodID(frame_class, "<init>", "([BZJ)V");
  if (constructor == nullptr) return nullptr;
  jobject frame = env->NewObject(frame_class, constructor, payload,
                                 keyframe ? JNI_TRUE : JNI_FALSE,
                                 static_cast<jlong>(pts_micros));
  env->DeleteLocalRef(payload);
  env->DeleteLocalRef(frame_class);
  return frame;
}

extern "C" JNIEXPORT void JNICALL
Java_expo_modules_mirrorxcapture_MirrorXNativeQueue_nativeDestroy(
    JNIEnv*, jclass, jlong handle) {
  if (handle != kInvalidHandle) mirrorx_h264_queue_free(from_handle(handle));
}
