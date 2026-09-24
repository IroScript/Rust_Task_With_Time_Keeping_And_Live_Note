package com.tasknote.task_note_mobile

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.provider.Settings
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val CHANNEL = "com.tasknote.task_note_mobile/overlay_permission"

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL).setMethodCallHandler { call, result ->
            when (call.method) {
                "isOverlayPermissionGranted" -> {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                        try {
                            result.success(Settings.canDrawOverlays(this))
                        } catch (e: Exception) {
                            result.success(false)
                        }
                    } else {
                        result.success(true)
                    }
                }
                "requestOverlayPermission" -> {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                        var success = false
                        // Tier 1: Try package-specific overlay permission intent
                        try {
                            val intent = Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION).apply {
                                data = Uri.parse("package:$packageName")
                                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                            }
                            startActivity(intent)
                            success = true
                        } catch (e1: Exception) {
                            // Tier 2: Try general overlay permission management screen
                            try {
                                val intent = Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION).apply {
                                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                }
                                startActivity(intent)
                                success = true
                            } catch (e2: Exception) {
                                // Tier 3: Fallback directly to App Details Settings
                                try {
                                    val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                                        data = Uri.parse("package:$packageName")
                                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                    }
                                    startActivity(intent)
                                    success = true
                                } catch (e3: Exception) {
                                    success = false
                                }
                            }
                        }
                        result.success(success)
                    } else {
                        result.success(true)
                    }
                }
                "openAppDetailsSettings" -> {
                    try {
                        val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                            data = Uri.parse("package:$packageName")
                            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        }
                        startActivity(intent)
                        result.success(true)
                    } catch (e: Exception) {
                        result.success(false)
                    }
                }
                "openOverlaySettings" -> {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                        var success = false
                        try {
                            val intent = Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION).apply {
                                data = Uri.parse("package:$packageName")
                                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                            }
                            startActivity(intent)
                            success = true
                        } catch (e1: Exception) {
                            try {
                                val intent = Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION).apply {
                                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                }
                                startActivity(intent)
                                success = true
                            } catch (e2: Exception) {
                                success = false
                            }
                        }
                        result.success(success)
                    } else {
                        result.success(true)
                    }
                }
                "showNativeFloatingWindow" -> {
                    val title = call.argument<String>("title") ?: "Task & Note"
                    val subtitle = call.argument<String>("subtitle") ?: ""
                    val note = call.argument<String>("note") ?: ""
                    val seconds = call.argument<Int>("seconds") ?: 0
                    val running = call.argument<Boolean>("isRunning") ?: true

                    try {
                        val intent = Intent(this, FloatingWindowService::class.java).apply {
                            action = FloatingWindowService.ACTION_UPDATE
                            putExtra(FloatingWindowService.EXTRA_TITLE, title)
                            putExtra(FloatingWindowService.EXTRA_SUBTITLE, subtitle)
                            putExtra(FloatingWindowService.EXTRA_NOTE, note)
                            putExtra(FloatingWindowService.EXTRA_SECONDS, seconds)
                            putExtra(FloatingWindowService.EXTRA_IS_RUNNING, running)
                        }
                        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                            startForegroundService(intent)
                        } else {
                            startService(intent)
                        }
                        result.success(true)
                    } catch (e: Exception) {
                        result.success(false)
                    }
                }
                "closeNativeFloatingWindow" -> {
                    try {
                        val intent = Intent(this, FloatingWindowService::class.java).apply {
                            action = FloatingWindowService.ACTION_STOP
                        }
                        stopService(intent)
                        result.success(true)
                    } catch (e: Exception) {
                        result.success(false)
                    }
                }
                "isNativeFloatingWindowActive" -> {
                    result.success(FloatingWindowService.isRunning)
                }
                "updateFloatingWindowData" -> {
                    val title = call.argument<String>("title") ?: "Task & Note"
                    val subtitle = call.argument<String>("subtitle") ?: ""
                    val note = call.argument<String>("note") ?: ""
                    val seconds = call.argument<Int>("seconds") ?: 0
                    val running = call.argument<Boolean>("isRunning") ?: true
                    FloatingWindowService.updateData(this, title, subtitle, note, seconds, running)
                    result.success(true)
                }
                else -> result.notImplemented()
            }
        }
    }
}
