#ifndef MIRRORX_MEDIA_H
#define MIRRORX_MEDIA_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MirrorXH264QueueHandle MirrorXH264QueueHandle;

MirrorXH264QueueHandle *mirrorx_h264_queue_new(size_t capacity);
int32_t mirrorx_h264_queue_push(
    MirrorXH264QueueHandle *handle,
    const uint8_t *data,
    size_t len,
    bool keyframe,
    uint64_t pts_micros
);
int32_t mirrorx_h264_queue_pop(
    MirrorXH264QueueHandle *handle,
    uint8_t *out,
    size_t out_capacity,
    size_t *out_len,
    bool *out_keyframe,
    uint64_t *out_pts_micros
);
void mirrorx_h264_queue_free(MirrorXH264QueueHandle *handle);

enum {
    MIRRORX_H264_QUEUE_EMPTY = 0,
    MIRRORX_H264_QUEUE_OK = 1,
    MIRRORX_H264_QUEUE_INVALID = -1,
    MIRRORX_H264_QUEUE_BUFFER_TOO_SMALL = -2
};

#ifdef __cplusplus
}
#endif

#endif
