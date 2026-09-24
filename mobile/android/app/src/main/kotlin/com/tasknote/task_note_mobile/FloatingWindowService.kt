package com.tasknote.task_note_mobile

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import androidx.core.app.NotificationCompat

class FloatingWindowService : Service() {

    private var windowManager: WindowManager? = null
    private var floatingView: View? = null
    private var layoutParams: WindowManager.LayoutParams? = null

    private var titleTextView: TextView? = null
    private var subtitleTextView: TextView? = null
    private var stopwatchTextView: TextView? = null
    private var playPauseButton: TextView? = null

    private var stopwatchSeconds = 0
    private var isStopwatchRunning = false
    private val mainHandler = Handler(Looper.getMainLooper())
    private val tickerRunnable = object : Runnable {
        override fun run() {
            if (isStopwatchRunning) {
                stopwatchSeconds++
                updateStopwatchDisplay()
            }
            mainHandler.postDelayed(this, 1000)
        }
    }

    companion object {
        const val CHANNEL_ID = "floating_task_window_channel"
        const val NOTIFICATION_ID = 4580
        var isRunning = false
        var currentInstance: FloatingWindowService? = null

        const val ACTION_START = "ACTION_START"
        const val ACTION_STOP = "ACTION_STOP"
        const val ACTION_UPDATE = "ACTION_UPDATE"

        const val EXTRA_TITLE = "EXTRA_TITLE"
        const val EXTRA_SUBTITLE = "EXTRA_SUBTITLE"
        const val EXTRA_NOTE = "EXTRA_NOTE"
        const val EXTRA_SECONDS = "EXTRA_SECONDS"
        const val EXTRA_IS_RUNNING = "EXTRA_IS_RUNNING"

        fun updateData(context: Context, title: String, subtitle: String, note: String, seconds: Int, running: Boolean) {
            val intent = Intent(context, FloatingWindowService::class.java).apply {
                action = ACTION_UPDATE
                putExtra(EXTRA_TITLE, title)
                putExtra(EXTRA_SUBTITLE, subtitle)
                putExtra(EXTRA_NOTE, note)
                putExtra(EXTRA_SECONDS, seconds)
                putExtra(EXTRA_IS_RUNNING, running)
            }
            if (isRunning) {
                currentInstance?.applyData(title, subtitle, note, seconds, running)
            } else {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    context.startForegroundService(intent)
                } else {
                    context.startService(intent)
                }
            }
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        currentInstance = this
        isRunning = true
        createNotificationChannel()
        startForeground(NOTIFICATION_ID, createNotification("Task & Note Active", "Floating Widget Running"))
        createFloatingWindow()
        mainHandler.post(tickerRunnable)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent != null) {
            when (intent.action) {
                ACTION_STOP -> {
                    stopSelf()
                    return START_NOT_STICKY
                }
                ACTION_UPDATE -> {
                    val title = intent.getStringExtra(EXTRA_TITLE) ?: "Task & Note"
                    val subtitle = intent.getStringExtra(EXTRA_SUBTITLE) ?: ""
                    val note = intent.getStringExtra(EXTRA_NOTE) ?: ""
                    val seconds = intent.getIntExtra(EXTRA_SECONDS, stopwatchSeconds)
                    val running = intent.getBooleanExtra(EXTRA_IS_RUNNING, isStopwatchRunning)
                    applyData(title, subtitle, note, seconds, running)
                }
            }
        }
        return START_STICKY
    }

    private fun dpToPx(dp: Int): Int {
        return (dp * resources.displayMetrics.density).toInt()
    }

    private fun createFloatingWindow() {
        windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager

        val windowType = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
        } else {
            @Suppress("DEPRECATION")
            WindowManager.LayoutParams.TYPE_PHONE
        }

        layoutParams = WindowManager.LayoutParams(
            dpToPx(280),
            WindowManager.LayoutParams.WRAP_CONTENT,
            windowType,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                    WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or
                    WindowManager.LayoutParams.FLAG_HARDWARE_ACCELERATED,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = dpToPx(30)
            y = dpToPx(120)
        }

        // Build the Cyberpunk UI Programmatically
        val rootLayout = FrameLayout(this)

        val cardLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dpToPx(14), dpToPx(12), dpToPx(14), dpToPx(12))

            // Background with Neon Border
            val bg = GradientDrawable().apply {
                setColor(Color.parseColor("#E60D121F")) // Dark cosmic blue 90% opacity
                cornerRadius = dpToPx(16).toFloat()
                setStroke(dpToPx(2), Color.parseColor("#00F0FF")) // Neon Cyan border
            }
            background = bg
            elevation = dpToPx(8).toFloat()
        }

        // 1. Header Bar (Glowing indicator + Drag Handle + Close Button)
        val headerLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        val glowIndicator = View(this).apply {
            val indicatorBg = GradientDrawable().apply {
                shape = GradientDrawable.OVAL
                setColor(Color.parseColor("#39FF14")) // Neon Lime
            }
            background = indicatorBg
            layoutParams = LinearLayout.LayoutParams(dpToPx(8), dpToPx(8)).apply {
                marginEnd = dpToPx(8)
            }
        }

        val headerTitle = TextView(this).apply {
            text = "⚡ TASK & LIVE NOTE"
            textSize = 10f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#00F0FF"))
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1.0f)
        }

        // Open App Button
        val openAppBtn = TextView(this).apply {
            text = "↗"
            textSize = 14f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#39FF14"))
            setPadding(dpToPx(6), dpToPx(2), dpToPx(6), dpToPx(2))
            setOnClickListener {
                val launchIntent = packageManager.getLaunchIntentForPackage(packageName)?.apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT)
                }
                if (launchIntent != null) {
                    startActivity(launchIntent)
                }
            }
        }

        // Close Button (✕)
        val closeBtn = TextView(this).apply {
            text = "✕"
            textSize = 14f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#FF007F")) // Neon Pink
            setPadding(dpToPx(8), dpToPx(2), dpToPx(4), dpToPx(2))
            setOnClickListener {
                stopSelf()
            }
        }

        headerLayout.addView(glowIndicator)
        headerLayout.addView(headerTitle)
        headerLayout.addView(openAppBtn)
        headerLayout.addView(closeBtn)
        cardLayout.addView(headerLayout)

        // 2. Main Task Title
        titleTextView = TextView(this).apply {
            text = "Focus on the work - Success is near"
            textSize = 14f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.WHITE)
            maxLines = 2
            setPadding(0, dpToPx(8), 0, dpToPx(2))
        }
        cardLayout.addView(titleTextView)

        // 3. Auxiliary / Live Note Text
        subtitleTextView = TextView(this).apply {
            text = "Keep pushing - You're doing great! ✨"
            textSize = 11f
            setTextColor(Color.parseColor("#A0AEC0"))
            maxLines = 2
            setPadding(0, 0, 0, dpToPx(8))
        }
        cardLayout.addView(subtitleTextView)

        // 4. Timer & Controls Bar
        val timerLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            val timerBg = GradientDrawable().apply {
                setColor(Color.parseColor("#3300F0FF"))
                cornerRadius = dpToPx(8).toFloat()
            }
            background = timerBg
            setPadding(dpToPx(10), dpToPx(6), dpToPx(10), dpToPx(6))
        }

        stopwatchTextView = TextView(this).apply {
            text = "⏱ 00:00:00"
            textSize = 13f
            typeface = Typeface.MONOSPACE
            setTextColor(Color.parseColor("#00F0FF"))
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1.0f)
        }

        playPauseButton = TextView(this).apply {
            text = "▶"
            textSize = 13f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#39FF14"))
            setPadding(dpToPx(8), 0, dpToPx(4), 0)
            setOnClickListener {
                isStopwatchRunning = !isStopwatchRunning
                text = if (isStopwatchRunning) "⏸" else "▶"
                glowIndicator.background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(if (isStopwatchRunning) Color.parseColor("#39FF14") else Color.parseColor("#FF9900"))
                }
            }
        }

        timerLayout.addView(stopwatchTextView)
        timerLayout.addView(playPauseButton)
        cardLayout.addView(timerLayout)

        rootLayout.addView(cardLayout)

        // Smooth Drag Handling
        var initialX = 0
        var initialY = 0
        var initialTouchX = 0f
        var initialTouchY = 0f

        cardLayout.setOnTouchListener { _, event ->
            val params = layoutParams ?: return@setOnTouchListener false
            when (event.action) {
                MotionEvent.ACTION_DOWN -> {
                    initialX = params.x
                    initialY = params.y
                    initialTouchX = event.rawX
                    initialTouchY = event.rawY
                    true
                }
                MotionEvent.ACTION_MOVE -> {
                    val deltaX = (event.rawX - initialTouchX).toInt()
                    val deltaY = (event.rawY - initialTouchY).toInt()
                    params.x = initialX + deltaX
                    params.y = initialY + deltaY
                    windowManager?.updateViewLayout(floatingView, params)
                    true
                }
                else -> false
            }
        }

        floatingView = rootLayout
        try {
            windowManager?.addView(floatingView, layoutParams)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    fun applyData(title: String, subtitle: String, note: String, seconds: Int, running: Boolean) {
        mainHandler.post {
            titleTextView?.text = title
            subtitleTextView?.text = if (note.isNotBlank()) note else subtitle
            stopwatchSeconds = seconds
            isStopwatchRunning = running
            playPauseButton?.text = if (running) "⏸" else "▶"
            updateStopwatchDisplay()
        }
    }

    private fun updateStopwatchDisplay() {
        val h = stopwatchSeconds / 3600
        val m = (stopwatchSeconds % 3600) / 60
        val s = stopwatchSeconds % 60
        val str = if (h > 0) {
            String.format("⏱ %02d:%02d:%02d", h, m, s)
        } else {
            String.format("⏱ %02d:%02d", m, s)
        }
        stopwatchTextView?.text = str
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Task & Note Floating Window",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Shows live status of active task floating window"
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(title: String, content: String): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            packageManager.getLaunchIntentForPackage(packageName),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) PendingIntent.FLAG_IMMUTABLE else 0
        )

        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle(title)
            .setContentText(content)
            .setSmallIcon(android.R.drawable.ic_menu_agenda)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    override fun onDestroy() {
        isRunning = false
        currentInstance = null
        mainHandler.removeCallbacks(tickerRunnable)
        if (floatingView != null && windowManager != null) {
            try {
                windowManager?.removeView(floatingView)
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
        super.onDestroy()
    }
}
