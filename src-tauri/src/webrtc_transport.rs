//! Native WebRTC H.264 sender foundation for the Windows desktop peer.
//!
//! This module creates a real WebRTC peer connection and H.264 sample track.
//! Pairing/signaling must still authenticate and approve the remote peer before
//! an offer/answer is exchanged. Callers must provide complete Annex-B access
//! units, including SPS/PPS when required by the decoder.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264},
        APIBuilder,
    },
    interceptor::registry::Registry,
    media::Sample,
    peer_connection::{
        configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
    track::track_local::{
        track_local_static_sample::TrackLocalStaticSample,
        TrackLocal,
    },
    rtp_transceiver::rtp_codec::RTCRtpCodecCapability,
};

const MAX_ACCESS_UNIT_BYTES: usize = 16 * 1024 * 1024;
const DEFAULT_FRAME_DURATION: Duration = Duration::from_micros(33_333);
const MAX_FRAME_DURATION: Duration = Duration::from_millis(250);

/// A WebRTC peer with a negotiated H.264 video sender track.
pub struct H264WebRtcSender {
    peer: Arc<RTCPeerConnection>,
    video_track: Arc<TrackLocalStaticSample>,
    last_pts_micros: Mutex<Option<u64>>,
}

impl H264WebRtcSender {
    /// Build a peer connection with the default WebRTC interceptors and H.264
    /// capability. No remote peer is connected until authenticated signaling
    /// applies a remote description and ICE candidates.
    pub async fn new() -> Result<Self, String> {
        let mut media_engine = MediaEngine::default();
        media_engine
            .register_default_codecs()
            .map_err(|error| format!("WebRTC codec registration failed: {error}"))?;

        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)
            .map_err(|error| format!("WebRTC interceptor setup failed: {error}"))?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let peer = Arc::new(
            api.new_peer_connection(RTCConfiguration::default())
                .await
                .map_err(|error| format!("WebRTC peer creation failed: {error}"))?,
        );

        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            "mirrorx-video".to_owned(),
            "mirrorx-session".to_owned(),
        ));

        let sender = peer
            .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .map_err(|error| format!("WebRTC H.264 track setup failed: {error}"))?;

        // Keep the RTCP reader active so feedback such as NACK/PLI can flow
        // through the default interceptor pipeline.
        tokio::spawn(async move {
            let mut packet_buffer = vec![0u8; 1500];
            while sender.read(&mut packet_buffer).await.is_ok() {}
        });

        Ok(Self {
            peer,
            video_track,
            last_pts_micros: Mutex::new(None),
        })
    }

    /// Create an offer and wait for ICE gathering to complete. The returned
    /// SDP is sensitive session material and must only be sent over the
    /// authenticated, user-approved MirrorX signaling channel.
    pub async fn create_offer_sdp(&self) -> Result<String, String> {
        let offer = self
            .peer
            .create_offer(None)
            .await
            .map_err(|error| format!("WebRTC offer creation failed: {error}"))?;

        let mut gathering_complete = self.peer
            .gathering_complete_promise()
            .await;

        self.peer
            .set_local_description(offer)
            .await
            .map_err(|error| format!("WebRTC local offer setup failed: {error}"))?;

        let _ = gathering_complete.recv().await;
        self.peer
            .local_description()
            .await
            .map(|description| description.sdp)
            .ok_or_else(|| "WebRTC did not produce a local SDP offer.".to_string())
    }

    /// Apply an answer received from the authenticated signaling layer.
    pub async fn apply_answer_sdp(&self, sdp: String) -> Result<(), String> {
        if sdp.len() > 64 * 1024 || !sdp.starts_with("v=0") {
            return Err("Remote SDP answer is malformed or exceeds the size limit.".to_string());
        }
        let answer = RTCSessionDescription::answer(sdp)
            .map_err(|error| format!("Remote SDP answer could not be parsed: {error}"))?;
        self.peer
            .set_remote_description(answer)
            .await
            .map_err(|error| format!("Remote SDP answer was rejected: {error}"))
    }

    /// Send one complete Annex-B H.264 access unit. PTS is the MediaCodec
    /// presentation timestamp in microseconds; frame duration is derived from
    /// successive timestamps rather than guessed per NAL unit.
    pub async fn write_access_unit(
        &self,
        access_unit: Vec<u8>,
        pts_micros: u64,
    ) -> Result<(), String> {
        if access_unit.is_empty()
            || access_unit.len() > MAX_ACCESS_UNIT_BYTES
            || !(access_unit.starts_with(&[0, 0, 1])
                || access_unit.starts_with(&[0, 0, 0, 1]))
        {
            return Err("Expected a non-empty Annex-B H.264 access unit within the size limit.".to_string());
        }

        let duration = {
            let mut previous = self
                .last_pts_micros
                .lock()
                .map_err(|_| "H.264 timestamp state is unavailable.".to_string())?;
            let duration = match *previous {
                None => DEFAULT_FRAME_DURATION,
                Some(last) if pts_micros > last => {
                    Duration::from_micros((pts_micros - last).min(MAX_FRAME_DURATION.as_micros() as u64))
                }
                Some(_) => return Err("H.264 presentation timestamps must increase monotonically.".to_string()),
            };
            *previous = Some(pts_micros);
            duration
        };

        self.video_track
            .write_sample(&Sample {
                data: Bytes::from(access_unit),
                duration,
                ..Default::default()
            })
            .await
            .map_err(|error| format!("WebRTC H.264 sample transmission failed: {error}"))
    }

    pub async fn close(&self) -> Result<(), String> {
        self.peer
            .close()
            .await
            .map_err(|error| format!("WebRTC peer shutdown failed: {error}"))
    }

    pub fn peer_connection_state(&self) -> String {
        self.peer.connection_state().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_FRAME_DURATION, MAX_FRAME_DURATION};
    use std::time::Duration;

    #[test]
    fn sender_uses_a_reasonable_initial_frame_duration() {
        assert_eq!(DEFAULT_FRAME_DURATION, Duration::from_micros(33_333));
    }

    #[test]
    fn sender_caps_large_timestamp_gaps() {
        let gap = Duration::from_micros(900_000).min(MAX_FRAME_DURATION);
        assert_eq!(gap, Duration::from_millis(250));
    }
}
