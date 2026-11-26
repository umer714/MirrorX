import { useEffect, useRef, useState } from "react";

type Props = {
  peerConnection: RTCPeerConnection | null;
  className?: string;
};

/**
 * Renders the remote WebRTC video track inside the desktop workspace.
 * Pairing/signaling must create and authenticate the peer connection before
 * passing it here. This component never creates an unauthenticated connection.
 */
export default function MirrorXVideoReceiver({ peerConnection, className }: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [status, setStatus] = useState("WAITING FOR VIDEO");
  const [muted, setMuted] = useState(true);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !peerConnection) {
      setStatus("WAITING FOR CONNECTION");
      return;
    }

    const stream = new MediaStream();
    const onTrack = (event: RTCTrackEvent) => {
      const incoming = event.streams[0];
      if (incoming) {
        incoming.getTracks().forEach((track) => {
          if (!stream.getTracks().some((current) => current.id === track.id)) stream.addTrack(track);
        });
      } else {
        stream.addTrack(event.track);
      }
      video.srcObject = stream;
      void video.play().catch(() => setStatus("TAP TO START VIDEO"));
      setStatus("VIDEO RECEIVING");
    };
    const onConnection = () => {
      const state = peerConnection.connectionState;
      if (state === "connected") setStatus("CONNECTED");
      else if (state === "failed") setStatus("CONNECTION FAILED");
      else if (state === "disconnected") setStatus("RECONNECTING");
      else if (state === "closed") setStatus("SESSION CLOSED");
    };
    peerConnection.addEventListener("track", onTrack);
    peerConnection.addEventListener("connectionstatechange", onConnection);
    onConnection();

    return () => {
      peerConnection.removeEventListener("track", onTrack);
      peerConnection.removeEventListener("connectionstatechange", onConnection);
      video.srcObject = null;
      stream.getTracks().forEach((track) => track.stop());
    };
  }, [peerConnection]);

  return (
    <section className={className ?? "mx-video-receiver"} aria-label="Phone screen preview">
      <div className="mx-video-receiver__topline">
        <span>LIVE DEVICE PREVIEW</span>
        <span role="status">{status}</span>
      </div>
      <div className="mx-video-receiver__stage">
        <video ref={videoRef} autoPlay playsInline muted={muted} />
        {!peerConnection && <div className="mx-video-receiver__empty">Pair an approved device to begin</div>}
        <button type="button" onClick={() => setMuted((value) => !value)} disabled={!peerConnection}>
          {muted ? "Unmute" : "Mute"}
        </button>
      </div>
    </section>
  );
}
