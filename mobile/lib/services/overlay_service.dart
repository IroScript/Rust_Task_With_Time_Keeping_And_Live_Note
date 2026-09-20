import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_overlay_window/flutter_overlay_window.dart';
import '../models/task_card.dart';

/// Service managing the System Floating Overlay Window (Chat Head / Floating Widget)
class OverlayService {
  static final OverlayService instance = OverlayService._internal();
  OverlayService._internal();

  bool _isOverlayActive = false;
  bool get isOverlayActive => _isOverlayActive;

  /// Check if the SYSTEM_ALERT_WINDOW permission is granted
  Future<bool> isPermissionGranted() async {
    try {
      final granted = await FlutterOverlayWindow.isPermissionGranted();
      return granted;
    } catch (e) {
      debugPrint('[OverlayService] Error checking permission: $e');
      return false;
    }
  }

  /// Open system settings to request "Display over other apps" permission
  Future<bool?> requestPermission() async {
    try {
      return await FlutterOverlayWindow.requestPermission();
    } catch (e) {
      debugPrint('[OverlayService] Error requesting permission: $e');
      return false;
    }
  }

  /// Launch the Floating Overlay Window with the given active card
  Future<bool> showFloatingOverlay({TaskCard? activeCard}) async {
    try {
      final granted = await isPermissionGranted();
      if (!granted) {
        final requested = await requestPermission();
        if (requested != true) {
          debugPrint('[OverlayService] Permission not granted by user');
          return false;
        }
      }

      await FlutterOverlayWindow.showOverlay(
        height: 180, // Compact collapsed floating pill
        width: 180,
        alignment: OverlayAlignment.topRight,
        visibility: NotificationVisibility.visibilityPublic,
        flag: OverlayFlag.defaultFlag,
        overlayTitle: "Task & Note Widget",
        overlayContent: activeCard?.mainText ?? "Floating Task & Live Note Active",
        enableDrag: true,
        positionGravity: PositionGravity.auto,
      );

      _isOverlayActive = true;

      // Share current card data to overlay
      if (activeCard != null) {
        await syncCardData(activeCard);
      }

      return true;
    } catch (e) {
      debugPrint('[OverlayService] Error launching overlay: $e');
      return false;
    }
  }

  /// Close / dismiss the floating overlay window
  Future<void> closeFloatingOverlay() async {
    try {
      await FlutterOverlayWindow.closeOverlay();
      _isOverlayActive = false;
    } catch (e) {
      debugPrint('[OverlayService] Error closing overlay: $e');
    }
  }

  /// Sync active TaskCard details into the floating window
  Future<void> syncCardData(TaskCard card) async {
    try {
      final payload = jsonEncode({
        'type': 'CARD_UPDATE',
        'id': card.id,
        'title': card.mainText,
        'subtitle': card.subText,
        'liveNote': card.liveNote,
        'stopwatchSeconds': card.stopwatchSeconds,
        'isStopwatchRunning': card.isStopwatchRunning,
        'clockMode': card.clockMode.label,
        'depth': card.depth,
      });
      await FlutterOverlayWindow.shareData(payload);
    } catch (e) {
      debugPrint('[OverlayService] Error sharing data to overlay: $e');
    }
  }

  /// Resize floating overlay dynamically (e.g. Expand to full card vs Collapse to floating pill)
  Future<void> resizeOverlay({required int width, required int height, bool enableDrag = true}) async {
    try {
      await FlutterOverlayWindow.resizeOverlay(width, height, enableDrag);
    } catch (e) {
      debugPrint('[OverlayService] Error resizing overlay: $e');
    }
  }
}
