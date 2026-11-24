import React, { useState } from "react";
import {
  SafeAreaView,
  ScrollView,
  StatusBar,
  StyleSheet,
  Text,
  Pressable,
  View,
  Platform,
} from "react-native";
import { requestScreenCapturePermission } from "./modules/mirrorx-capture/src";

type Mode = "sender" | "controller";

const palette = {
  background: "#090A12",
  surface: "#121421",
  surfaceRaised: "#191D2D",
  border: "#282D43",
  text: "#F4F5FC",
  muted: "#9299B2",
  accent: "#8B75FF",
  accentSoft: "#211C3D",
  warning: "#E6B86A",
};

function FeatureCard({
  icon,
  title,
  description,
  selected,
  onPress,
}: {
  icon: string;
  title: string;
  description: string;
  selected: boolean;
  onPress: () => void;
}) {
  return (
    <Pressable
      accessibilityRole="button"
      accessibilityState={{ selected }}
      onPress={onPress}
      style={[styles.featureCard, selected && styles.featureCardSelected]}
    >
      <View style={styles.featureTop}>
        <Text style={styles.featureIcon}>{icon}</Text>
        <View style={[styles.radio, selected && styles.radioSelected]}>
          {selected ? <View style={styles.radioDot} /> : null}
        </View>
      </View>
      <Text style={styles.featureTitle}>{title}</Text>
      <Text style={styles.featureDescription}>{description}</Text>
    </Pressable>
  );
}

export default function App() {
  const [mode, setMode] = useState<Mode>("sender");
  const [capturePermission, setCapturePermission] = useState(false);
  const [captureMessage, setCaptureMessage] = useState("");

  const requestCapturePermission = async () => {
    try {
      const result = await requestScreenCapturePermission();
      setCapturePermission(result.granted);
      setCaptureMessage(result.granted
        ? "Android permission approved. Live streaming is the next integration step."
        : "Permission was not granted. You can try again whenever you are ready.");
    } catch (error) {
      setCaptureMessage(error instanceof Error ? error.message : "Unable to request screen-capture permission.");
    }
  };

  return (
    <SafeAreaView style={styles.safeArea}>
      <StatusBar barStyle="light-content" />
      <ScrollView contentContainerStyle={styles.container}>
        <View style={styles.header}>
          <View style={styles.brandMark}>
            <Text style={styles.brandMarkText}>M</Text>
          </View>
          <View style={styles.brandCopy}>
            <Text style={styles.brandName}>MirrorX</Text>
            <Text style={styles.brandTagline}>YOUR PHONE. YOUR PC. ONE EXPERIENCE.</Text>
          </View>
          <View style={styles.statusPill}>
            <View style={styles.statusDot} />
            <Text style={styles.statusText}>LOCAL</Text>
          </View>
        </View>

        <View style={styles.hero}>
          <Text style={styles.eyebrow}>MOBILE COMPANION</Text>
          <Text style={styles.heroTitle}>Connect your{ "\n" }world.</Text>
          <Text style={styles.heroDescription}>
            Mirror your phone to your PC, or use your phone as a wireless
            controller for your computer.
          </Text>
        </View>

        <Text style={styles.sectionLabel}>CHOOSE YOUR MODE</Text>
        <View style={styles.cards}>
          <FeatureCard
            icon="▣"
            title="Screen Sender"
            description="Share your Android or iPhone screen with MirrorX on Windows."
            selected={mode === "sender"}
            onPress={() => setMode("sender")}
          />
          <FeatureCard
            icon="⌘"
            title="Remote Controller"
            description="Use your phone as a touchpad, keyboard and shortcut remote."
            selected={mode === "controller"}
            onPress={() => setMode("controller")}
          />
        </View>

        <View style={styles.connectionPanel}>
          <View style={styles.panelHeading}>
            <View>
              <Text style={styles.panelTitle}>
                {mode === "sender" ? "Connect to your PC" : "Find your PC"}
              </Text>
              <Text style={styles.panelSubtitle}>Same Wi-Fi network recommended</Text>
            </View>
            <View style={styles.wifiIcon}>
              <Text style={styles.wifiIconText}>⌁</Text>
            </View>
          </View>

          {mode === "sender" && Platform.OS === "android" ? (
            <View style={styles.capturePermissionPanel}>
              <Text style={styles.capturePermissionTitle}>ANDROID SCREEN PERMISSION</Text>
              <Text style={styles.capturePermissionDescription}>
                Start with Android's official consent screen. MirrorX cannot capture anything unless you approve it.
              </Text>
              <Pressable
                accessibilityRole="button"
                onPress={requestCapturePermission}
                style={styles.capturePermissionButton}
              >
                <Text style={styles.capturePermissionButtonText}>
                  {capturePermission ? "Permission approved" : "Request screen permission"}
                </Text>
              </Pressable>
              {captureMessage ? <Text style={styles.capturePermissionMessage}>{captureMessage}</Text> : null}
            </View>
          ) : null}
          <View style={styles.emptyState}>
            <View style={styles.scanIcon}>
              <Text style={styles.scanIconText}>⌕</Text>
            </View>
            <Text style={styles.emptyTitle}>Desktop discovery is not connected yet</Text>
            <Text style={styles.emptyDescription}>
              The mobile interface is ready. Pairing, live streaming and remote
              input will activate when the desktop connection service is added.
            </Text>
          </View>
          <View style={styles.notice}>
            <Text style={styles.noticeIcon}>i</Text>
            <Text style={styles.noticeText}>
              Screen sharing always requires your operating system's permission.
              Nothing is streamed until you explicitly start a session.
            </Text>
          </View>
        </View>

        <View style={styles.footer}>
          <Text style={styles.footerTitle}>PRIVATE BY DESIGN</Text>
          <Text style={styles.footerText}>
            Local-network first. No account required for basic device pairing.
            Your screen is not uploaded to a MirrorX cloud service.
          </Text>
          <Text style={styles.version}>MIRRORX MOBILE · 0.1.0 · FOUNDATION BUILD</Text>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safeArea: { flex: 1, backgroundColor: palette.background },
  container: { paddingHorizontal: 22, paddingTop: 18, paddingBottom: 36 },
  header: { flexDirection: "row", alignItems: "center" },
  brandMark: {
    width: 42, height: 42, borderRadius: 14, backgroundColor: palette.accent,
    alignItems: "center", justifyContent: "center",
  },
  brandMarkText: { color: "#FFFFFF", fontSize: 24, fontWeight: "900" },
  brandCopy: { marginLeft: 11, flex: 1 },
  brandName: { color: palette.text, fontSize: 19, fontWeight: "800", letterSpacing: 0.2 },
  brandTagline: { color: palette.muted, fontSize: 8, letterSpacing: 1.15, marginTop: 3 },
  statusPill: {
    flexDirection: "row", alignItems: "center", paddingHorizontal: 10,
    paddingVertical: 7, borderRadius: 20, backgroundColor: palette.surface,
    borderWidth: 1, borderColor: palette.border,
  },
  statusDot: { width: 6, height: 6, borderRadius: 3, backgroundColor: "#78A9FF", marginRight: 6 },
  statusText: { color: palette.muted, fontSize: 9, fontWeight: "800", letterSpacing: 1 },
  hero: { marginTop: 42, marginBottom: 30 },
  eyebrow: { color: palette.accent, fontSize: 10, fontWeight: "800", letterSpacing: 2 },
  heroTitle: { color: palette.text, fontSize: 43, lineHeight: 48, fontWeight: "800", marginTop: 12 },
  heroDescription: { color: palette.muted, fontSize: 14, lineHeight: 22, marginTop: 13, maxWidth: 330 },
  sectionLabel: { color: palette.muted, fontSize: 10, fontWeight: "800", letterSpacing: 1.7, marginBottom: 12 },
  cards: { flexDirection: "row", gap: 10 },
  featureCard: {
    flex: 1, minHeight: 174, padding: 14, borderRadius: 18,
    backgroundColor: palette.surface, borderWidth: 1, borderColor: palette.border,
  },
  featureCardSelected: { borderColor: palette.accent, backgroundColor: palette.accentSoft },
  featureTop: { flexDirection: "row", justifyContent: "space-between", alignItems: "center", marginBottom: 18 },
  featureIcon: { color: palette.accent, fontSize: 25, fontWeight: "700" },
  radio: { width: 17, height: 17, borderRadius: 9, borderWidth: 1.5, borderColor: palette.muted, alignItems: "center", justifyContent: "center" },
  radioSelected: { borderColor: palette.accent },
  radioDot: { width: 8, height: 8, borderRadius: 4, backgroundColor: palette.accent },
  featureTitle: { color: palette.text, fontSize: 14, fontWeight: "800", marginBottom: 7 },
  featureDescription: { color: palette.muted, fontSize: 11, lineHeight: 17 },
  connectionPanel: {
    marginTop: 22, padding: 17, borderRadius: 20, backgroundColor: palette.surface,
    borderWidth: 1, borderColor: palette.border,
  },
  panelHeading: { flexDirection: "row", alignItems: "center", justifyContent: "space-between" },
  panelTitle: { color: palette.text, fontSize: 16, fontWeight: "800" },
  panelSubtitle: { color: palette.muted, fontSize: 11, marginTop: 5 },
  wifiIcon: { width: 36, height: 36, borderRadius: 12, backgroundColor: palette.surfaceRaised, alignItems: "center", justifyContent: "center" },
  wifiIconText: { color: palette.accent, fontSize: 25 },
  capturePermissionPanel: { marginTop: 18, padding: 14, borderRadius: 14, backgroundColor: palette.surfaceRaised, borderWidth: 1, borderColor: palette.border },
  capturePermissionTitle: { color: palette.accent, fontSize: 9, fontWeight: "900", letterSpacing: 1.3 },
  capturePermissionDescription: { color: palette.muted, fontSize: 11, lineHeight: 17, marginTop: 7 },
  capturePermissionButton: { marginTop: 12, paddingVertical: 12, paddingHorizontal: 14, borderRadius: 10, backgroundColor: palette.accent, alignItems: "center" },
  capturePermissionButtonText: { color: "#FFFFFF", fontSize: 12, fontWeight: "800" },
  capturePermissionMessage: { color: palette.muted, fontSize: 10, lineHeight: 15, marginTop: 9 },
  emptyState: { alignItems: "center", paddingHorizontal: 10, paddingTop: 28, paddingBottom: 24 },
  scanIcon: { width: 54, height: 54, borderRadius: 18, backgroundColor: palette.surfaceRaised, alignItems: "center", justifyContent: "center", marginBottom: 14 },
  scanIconText: { color: palette.accent, fontSize: 34 },
  emptyTitle: { color: palette.text, fontSize: 13, fontWeight: "800", textAlign: "center" },
  emptyDescription: { color: palette.muted, fontSize: 11, lineHeight: 17, textAlign: "center", marginTop: 8 },
  notice: { flexDirection: "row", alignItems: "flex-start", padding: 12, borderRadius: 12, backgroundColor: "#211D17" },
  noticeIcon: { color: palette.warning, fontWeight: "900", marginRight: 9, fontSize: 13 },
  noticeText: { color: "#C8B99D", fontSize: 10, lineHeight: 16, flex: 1 },
  footer: { marginTop: 28, paddingHorizontal: 3 },
  footerTitle: { color: palette.accent, fontSize: 9, fontWeight: "900", letterSpacing: 1.8 },
  footerText: { color: palette.muted, fontSize: 11, lineHeight: 17, marginTop: 8 },
  version: { color: "#5D637A", fontSize: 8, letterSpacing: 1, marginTop: 18 },
});