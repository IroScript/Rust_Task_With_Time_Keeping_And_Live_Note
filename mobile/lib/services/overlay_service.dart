import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_overlay_window/flutter_overlay_window.dart';
import '../models/task_card.dart';

/// Service managing the System Floating Overlay Window (Chat Head / Floating Widget)
class OverlayService {
  static final OverlayService instance = OverlayService._internal();
  OverlayService._internal();

  static const MethodChannel _nativeChannel =
      MethodChannel('com.tasknote.task_note_mobile/overlay_permission');

  bool _isOverlayActive = false;
  bool get isOverlayActive => _isOverlayActive;

  /// Check if the SYSTEM_ALERT_WINDOW permission is granted
  Future<bool> isPermissionGranted() async {
    try {
      final bool? nativeGranted =
          await _nativeChannel.invokeMethod<bool>('isOverlayPermissionGranted');
      if (nativeGranted != null) {
        return nativeGranted;
      }
    } catch (e) {
      debugPrint('[OverlayService] Native permission check error: $e');
    }
    try {
      final granted = await FlutterOverlayWindow.isPermissionGranted();
      return granted;
    } catch (e) {
      debugPrint('[OverlayService] FlutterOverlayWindow permission check error: $e');
      return false;
    }
  }

  /// Open system settings to request "Display over other apps" permission with 3-tier fallback
  Future<bool?> requestPermission() async {
    try {
      final bool? nativeSuccess =
          await _nativeChannel.invokeMethod<bool>('requestOverlayPermission');
      if (nativeSuccess == true) {
        return true;
      }
    } catch (e) {
      debugPrint('[OverlayService] Native permission request error: $e');
    }
    try {
      return await FlutterOverlayWindow.requestPermission();
    } catch (e) {
      debugPrint('[OverlayService] Error requesting permission: $e');
      return false;
    }
  }

  /// Open App Details Settings directly (fallback for restricted settings)
  Future<bool?> openAppDetailsSettings() async {
    try {
      return await _nativeChannel.invokeMethod<bool>('openAppDetailsSettings');
    } catch (e) {
      debugPrint('[OverlayService] Error opening app details: $e');
      return false;
    }
  }

  /// Open Overlay / Special App Access settings directly
  Future<bool?> openOverlaySettings() async {
    try {
      return await _nativeChannel.invokeMethod<bool>('openOverlaySettings');
    } catch (e) {
      debugPrint('[OverlayService] Error opening overlay settings: $e');
      return false;
    }
  }

  /// Check if the floating window is currently alive and active
  Future<bool> checkActualOverlayActive() async {
    try {
      final bool? active =
          await _nativeChannel.invokeMethod<bool>('isNativeFloatingWindowActive');
      if (active != null) {
        _isOverlayActive = active;
        return active;
      }
    } catch (_) {}
    return _isOverlayActive;
  }

  /// Launch the Floating Overlay Window with the given active card
  Future<bool> showFloatingOverlay({TaskCard? activeCard}) async {
    try {
      final granted = await isPermissionGranted();
      if (!granted) {
        await requestPermission();
        final recheck = await isPermissionGranted();
        if (!recheck) {
          debugPrint('[OverlayService] Permission not granted yet');
          return false;
        }
      }

      final title = activeCard?.mainText ?? "Daily Motivation Task & Note";
      final subtitle = activeCard?.subText ?? "Keep pushing forward! ✨";
      final note = activeCard?.liveNote ?? "";
      final seconds = activeCard?.stopwatchSeconds ?? 0;
      final running = activeCard?.isStopwatchRunning ?? true;

      final bool? success = await _nativeChannel.invokeMethod<bool>(
        'showNativeFloatingWindow',
        {
          'title': title,
          'subtitle': subtitle,
          'note': note,
          'seconds': seconds,
          'isRunning': running,
        },
      );

      _isOverlayActive = success ?? true;
      return _isOverlayActive;
    } catch (e) {
      debugPrint('[OverlayService] Error launching native floating overlay: $e');
      return false;
    }
  }

  /// Close / dismiss the floating overlay window
  Future<void> closeFloatingOverlay() async {
    try {
      await _nativeChannel.invokeMethod('closeNativeFloatingWindow');
      _isOverlayActive = false;
    } catch (e) {
      debugPrint('[OverlayService] Error closing native floating overlay: $e');
    }
  }

  /// Sync active TaskCard details into the floating window
  Future<void> syncCardData(TaskCard card) async {
    try {
      await _nativeChannel.invokeMethod('updateFloatingWindowData', {
        'title': card.mainText,
        'subtitle': card.subText,
        'note': card.liveNote,
        'seconds': card.stopwatchSeconds,
        'isRunning': card.isStopwatchRunning,
      });
    } catch (e) {
      debugPrint('[OverlayService] Error updating floating window data: $e');
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
