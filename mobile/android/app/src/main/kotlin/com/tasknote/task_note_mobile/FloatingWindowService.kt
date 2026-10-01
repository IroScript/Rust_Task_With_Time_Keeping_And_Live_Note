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
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.core.app.NotificationCompat

/// 100% Rust-Parity Android Native Draggable Floating Window Service
///
/// Implements 1:1 parity with:
/// - Rust card_header_widgets.rs:
///     * [+] Plus Button (22x22 dp, #3CB450 green border, #B21C1C red plus)
///     * 3 Clock Badges:
///         - Badge 0: Deadline (#A51616 Crimson Red, "12.10 PM")
///         - Badge 1: Sub-Task (#B95F0F Amber, "12.10 PM")
///         - Badge 2: Stopwatch (#1C76B9 Steel-Blue, "MM:SS", pulsing dot #28C8FF)
/// - Rust src/main.rs:2450-2540:
///     * Title bar auto-hide after 5.0s inactivity
///     * Toggle Panel / Close actions
/// - Rust src/views/live_note.rs:
///     * In-place Live Note viewing & quick writing
///     * Virtual scrolling indicator (>10 KB badge)
class FloatingWindowService : Service() {

    private var windowManager: WindowManager? = null
    private var floatingView: View? = null
    private var layoutParams: WindowManager.LayoutParams? = null

    // UI Elements
    private var titleTextView: TextView? = null
    private var subtitleTextView: TextView? = null
    private var liveNoteTextView: TextView? = null
    private var liveNoteEditLayout: LinearLayout? = null
    private var liveNoteEditText: EditText? = null
    private var virtualScrollBadge: TextView? = null

    // 3 Clock Badges
    private var badgeDeadlineTextView: TextView? = null
    private var badgeSubTaskTextView: TextView? = null
    private var badgeStopwatchTextView: TextView? = null
    private var pulsingDot: View? = null

    // Action buttons & Auto-hide
    private var actionButtonsLayout: LinearLayout? = null
    private var floatingButtonOpacity = 1.0f

    private var cardId = "1"
    private var depth = 0
    private var deadlineTime = "12.10 PM"
    private var subTaskTime = "12.10 PM"
    private var liveNoteText = ""
    private var noteTextSizeSp = 11.5f

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

    private val inactivityRunnable = Runnable {
        // 5.0s Inactivity fade matching Rust src/main.rs:2454
        floatingButtonOpacity = 0.0f
        actionButtonsLayout?.animate()?.alpha(0.0f)?.setDuration(300)?.start()
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
        const val EXTRA_CARD_ID = "EXTRA_CARD_ID"
        const val EXTRA_DEPTH = "EXTRA_DEPTH"
        const val EXTRA_DEADLINE = "EXTRA_DEADLINE"
        const val EXTRA_SUBTASK_TIME = "EXTRA_SUBTASK_TIME"

        fun updateData(context: Context, title: String, subtitle: String, note: String, seconds: Int, running: Boolean) {
            val intent = Intent(context, FloatingWindowService::class.java).apply {
                action = ACTION_UPDATE
                putExtra(EXTRA_TITLE, title)
                putExtra(EXTRA_SUBTITLE, subtitle)
                putExtra(EXTRA_NOTE, note)
                putExtra(EXTRA_SECONDS, seconds)
                putExtra(EXTRA_IS_RUNNING, running)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        isRunning = true
        currentInstance = this
        createNotificationChannel()
        val notification = createNotification("Task & Live Note Overlay", "Overlay is active")
        startForeground(NOTIFICATION_ID, notification)
        createFloatingWindow()
        mainHandler.post(tickerRunnable)
        resetInactivityTimer()
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
                    cardId = intent.getStringExtra(EXTRA_CARD_ID) ?: cardId
                    depth = intent.getIntExtra(EXTRA_DEPTH, depth)
                    deadlineTime = intent.getStringExtra(EXTRA_DEADLINE) ?: deadlineTime
                    subTaskTime = intent.getStringExtra(EXTRA_SUBTASK_TIME) ?: subTaskTime
                    applyData(title, subtitle, note, seconds, running)
                }
            }
        }
        return START_STICKY
    }

    private fun dpToPx(dp: Int): Int {
        return (dp * resources.displayMetrics.density).toInt()
    }

    private fun recordInteraction() {
        if (floatingButtonOpacity < 1.0f) {
            floatingButtonOpacity = 1.0f
            actionButtonsLayout?.animate()?.alpha(1.0f)?.setDuration(200)?.start()
        }
        resetInactivityTimer()
    }

    private fun resetInactivityTimer() {
        mainHandler.removeCallbacks(inactivityRunnable)
        mainHandler.postDelayed(inactivityRunnable, 5000)
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
            dpToPx(300),
            WindowManager.LayoutParams.WRAP_CONTENT,
            windowType,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                    WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN or
                    WindowManager.LayoutParams.FLAG_HARDWARE_ACCELERATED,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = dpToPx(24)
            y = dpToPx(100)
        }

        val rootLayout = FrameLayout(this)

        val cardLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dpToPx(12), dpToPx(10), dpToPx(12), dpToPx(10))

            // Holographic Biopunk Background (#080B10 with #3CB450 border)
            val bg = GradientDrawable().apply {
                setColor(Color.parseColor("#F2080B10"))
                cornerRadius = dpToPx(10).toFloat()
                setStroke(dpToPx(2), Color.parseColor("#3CB450")) // Exact green border
            }
            background = bg
            elevation = dpToPx(10).toFloat()
        }

        // ── 1. Top Bar: Glowing Indicator + Header Title + Auto-Hide Action Buttons ──
        val headerLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        val glowIndicator = View(this).apply {
            val indicatorBg = GradientDrawable().apply {
                shape = GradientDrawable.OVAL
                setColor(Color.parseColor("#3CB450"))
            }
            background = indicatorBg
            layoutParams = LinearLayout.LayoutParams(dpToPx(8), dpToPx(8)).apply {
                marginEnd = dpToPx(6)
            }
        }

        val headerTitle = TextView(this).apply {
            text = "⚡ TASK & LIVE NOTE"
            textSize = 9.5f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#3CB450"))
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1.0f)
        }

        // Floating Action Buttons (with 5.0s Auto-Hide Opacity Animation)
        actionButtonsLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        // Open App Button (↗)
        val openAppBtn = TextView(this).apply {
            text = "↗"
            textSize = 13f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#39FF14"))
            setPadding(dpToPx(6), dpToPx(2), dpToPx(6), dpToPx(2))
            setOnClickListener {
                recordInteraction()
                val launchIntent = packageManager.getLaunchIntentForPackage(packageName)?.apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT)
                }
                if (launchIntent != null) {
                    startActivity(launchIntent)
                }
            }
        }

        // Close Overlay Button (✕)
        val closeBtn = TextView(this).apply {
            text = "✕"
            textSize = 13f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#FF007F"))
            setPadding(dpToPx(6), dpToPx(2), dpToPx(2), dpToPx(2))
            setOnClickListener {
                stopSelf()
            }
        }

        actionButtonsLayout?.addView(openAppBtn)
        actionButtonsLayout?.addView(closeBtn)

        headerLayout.addView(glowIndicator)
        headerLayout.addView(headerTitle)
        headerLayout.addView(actionButtonsLayout)
        cardLayout.addView(headerLayout)

        // ── 2. The Holographic Header Row: [+] Plus Button + 3 Clock Badges ──
        val headerRowLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, dpToPx(6), 0, dpToPx(6))
        }

        // 2a. [+] Plus Button: 22x22 dp, green border (#3CB450), crimson red plus (#B21C1C)
        val plusButton = TextView(this).apply {
            text = "+"
            textSize = 15f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
            setTextColor(Color.parseColor("#B21C1C")) // Crimson Red Cross
            background = GradientDrawable().apply {
                cornerRadius = dpToPx(5).toFloat()
                setStroke(dpToPx(1), Color.parseColor("#3CB450"))
            }
            layoutParams = LinearLayout.LayoutParams(dpToPx(22), dpToPx(22)).apply {
                marginEnd = dpToPx(5)
            }
            setOnClickListener {
                recordInteraction()
                val newDepth = depth + 1
                Toast.makeText(this@FloatingWindowService, "Sub-card created (depth $newDepth, scale 95%)", Toast.LENGTH_SHORT).show()
                // Broadcast to Flutter app
                sendBroadcast(Intent("com.tasknote.ACTION_ADD_SUBCARD").apply {
                    putExtra("cardId", cardId)
                    putExtra("depth", newDepth)
                })
            }
        }
        headerRowLayout.addView(plusButton)

        // Helper to create pill clock badge chips (64x20 dp, 5 dp radius, #3CB450 border)
        fun createClockChip(textStr: String, textColorHex: String): TextView {
            return TextView(this).apply {
                text = textStr
                textSize = 10f
                typeface = Typeface.MONOSPACE
                setTextColor(Color.parseColor(textColorHex))
                gravity = Gravity.CENTER
                setPadding(dpToPx(6), dpToPx(2), dpToPx(6), dpToPx(2))
                background = GradientDrawable().apply {
                    cornerRadius = dpToPx(5).toFloat()
                    setStroke(dpToPx(1), Color.parseColor("#3CB450"))
                }
                layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, dpToPx(22)).apply {
                    marginEnd = dpToPx(4)
                }
            }
        }

        // 2b. Badge 0: Deadline (#A51616 Crimson Red)
        badgeDeadlineTextView = createClockChip(deadlineTime, "#A51616").apply {
            setOnClickListener {
                recordInteraction()
                Toast.makeText(this@FloatingWindowService, "Deadline: $deadlineTime", Toast.LENGTH_SHORT).show()
            }
        }
        headerRowLayout.addView(badgeDeadlineTextView)

        // 2c. Badge 1: Sub-Task Time (#B95F0F Amber)
        badgeSubTaskTextView = createClockChip(subTaskTime, "#B95F0F").apply {
            setOnClickListener {
                recordInteraction()
                Toast.makeText(this@FloatingWindowService, "Sub-task: $subTaskTime", Toast.LENGTH_SHORT).show()
            }
        }
        headerRowLayout.addView(badgeSubTaskTextView)

        // 2d. Badge 2: Stopwatch (#1C76B9 Steel-Blue) with pulsing blue dot
        val stopwatchBadgeContainer = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(dpToPx(6), dpToPx(2), dpToPx(6), dpToPx(2))
            background = GradientDrawable().apply {
                cornerRadius = dpToPx(5).toFloat()
                setStroke(dpToPx(1), Color.parseColor("#3CB450"))
            }
            layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, dpToPx(22))
        }

        badgeStopwatchTextView = TextView(this).apply {
            text = "00:00"
            textSize = 10f
            typeface = Typeface.MONOSPACE
            setTextColor(Color.parseColor("#1C76B9"))
        }

        pulsingDot = View(this).apply {
            background = GradientDrawable().apply {
                shape = GradientDrawable.OVAL
                setColor(Color.parseColor("#28C8FF"))
            }
            visibility = View.GONE
            layoutParams = LinearLayout.LayoutParams(dpToPx(5), dpToPx(5)).apply {
                marginStart = dpToPx(4)
            }
        }

        stopwatchBadgeContainer.addView(badgeStopwatchTextView)
        stopwatchBadgeContainer.addView(pulsingDot)

        stopwatchBadgeContainer.setOnClickListener {
            recordInteraction()
            isStopwatchRunning = !isStopwatchRunning
            pulsingDot?.visibility = if (isStopwatchRunning) View.VISIBLE else View.GONE
            updateStopwatchDisplay()
        }

        stopwatchBadgeContainer.setOnLongClickListener {
            recordInteraction()
            isStopwatchRunning = false
            stopwatchSeconds = 0
            pulsingDot?.visibility = View.GONE
            updateStopwatchDisplay()
            Toast.makeText(this@FloatingWindowService, "Stopwatch reset to 00:00", Toast.LENGTH_SHORT).show()
            true
        }

        headerRowLayout.addView(stopwatchBadgeContainer)
        cardLayout.addView(headerRowLayout)

        // ── 3. Main Task Content & Subtitle ──
        titleTextView = TextView(this).apply {
            text = "Focus on the work - Success is near"
            textSize = 13.5f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.WHITE)
            maxLines = 2
            setPadding(0, dpToPx(4), 0, dpToPx(2))
        }
        cardLayout.addView(titleTextView)

        subtitleTextView = TextView(this).apply {
            text = "Keep pushing forward! ✨"
            textSize = 10f
            setTextColor(Color.parseColor("#8B949E"))
            maxLines = 1
            setPadding(0, 0, 0, dpToPx(6))
        }
        cardLayout.addView(subtitleTextView)

        // ── 4. Virtual Scrolling Badge (>10 KB threshold) ──
        virtualScrollBadge = TextView(this).apply {
            text = "📄 Large Text (>10 KB) - Virtual Scrolling Active"
            textSize = 8.5f
            typeface = Typeface.MONOSPACE
            setTextColor(Color.parseColor("#64C8FF"))
            visibility = View.GONE
            setPadding(0, 0, 0, dpToPx(4))
        }
        cardLayout.addView(virtualScrollBadge)

        // ── 5. Live Note Box & In-Place Writing (src/views/live_note.rs) ──
        val liveNoteBox = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(dpToPx(8), dpToPx(6), dpToPx(8), dpToPx(6))
            background = GradientDrawable().apply {
                setColor(Color.parseColor("#FF090D14"))
                cornerRadius = dpToPx(6).toFloat()
                setStroke(dpToPx(1), Color.parseColor("#809D4EDD")) // #9D4EDD Neon Purple 50%
            }
        }

        val noteHeader = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        val noteTitle = TextView(this).apply {
            text = "📄 LIVE NOTE"
            textSize = 9f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#9D4EDD"))
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1.0f)
        }

        val fontPlusBtn = TextView(this).apply {
            text = "A+ "
            textSize = 9.5f
            setTextColor(Color.parseColor("#A0AEC0"))
            setOnClickListener {
                recordInteraction()
                noteTextSizeSp = (noteTextSizeSp + 1f).coerceIn(9f, 18f)
                liveNoteTextView?.textSize = noteTextSizeSp
                liveNoteEditText?.textSize = noteTextSizeSp
            }
        }

        val fontMinusBtn = TextView(this).apply {
            text = "A- "
            textSize = 9.5f
            setTextColor(Color.parseColor("#A0AEC0"))
            setOnClickListener {
                recordInteraction()
                noteTextSizeSp = (noteTextSizeSp - 1f).coerceIn(9f, 18f)
                liveNoteTextView?.textSize = noteTextSizeSp
                liveNoteEditText?.textSize = noteTextSizeSp
            }
        }

        val editToggleBtn = TextView(this).apply {
            text = "EDIT"
            textSize = 8.5f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#39FF14"))
            setPadding(dpToPx(6), dpToPx(1), dpToPx(6), dpToPx(1))
            background = GradientDrawable().apply {
                cornerRadius = dpToPx(3).toFloat()
                setStroke(dpToPx(1), Color.parseColor("#39FF14"))
            }
            setOnClickListener {
                recordInteraction()
                val isEditing = liveNoteEditLayout?.visibility == View.VISIBLE
                if (isEditing) {
                    // Save note
                    val updated = liveNoteEditText?.text?.toString()?.trim() ?: ""
                    liveNoteText = if (updated.isNotBlank()) updated else "No notes recorded."
                    liveNoteTextView?.text = liveNoteText
                    liveNoteEditLayout?.visibility = View.GONE
                    liveNoteTextView?.visibility = View.VISIBLE
                    text = "EDIT"
                    virtualScrollBadge?.visibility = if (liveNoteText.length > 10240) View.VISIBLE else View.GONE
                    Toast.makeText(this@FloatingWindowService, "Live note saved", Toast.LENGTH_SHORT).show()
                } else {
                    // Open inline editor
                    liveNoteEditText?.setText(if (liveNoteText == "No notes recorded.") "" else liveNoteText)
                    liveNoteTextView?.visibility = View.GONE
                    liveNoteEditLayout?.visibility = View.VISIBLE
                    text = "SAVE"
                }
            }
        }

        noteHeader.addView(noteTitle)
        noteHeader.addView(fontPlusBtn)
        noteHeader.addView(fontMinusBtn)
        noteHeader.addView(editToggleBtn)
        liveNoteBox.addView(noteHeader)

        liveNoteTextView = TextView(this).apply {
            text = "No notes recorded yet."
            textSize = noteTextSizeSp
            setTextColor(Color.parseColor("#E6EDF3"))
            maxLines = 3
            setPadding(0, dpToPx(4), 0, 0)
        }
        liveNoteBox.addView(liveNoteTextView)

        // In-line Edit layout
        liveNoteEditLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            visibility = View.GONE
            setPadding(0, dpToPx(4), 0, 0)
        }

        liveNoteEditText = EditText(this).apply {
            hint = "Type new live note..."
            setHintTextColor(Color.parseColor("#555555"))
            setTextColor(Color.WHITE)
            textSize = noteTextSizeSp
            maxLines = 3
            background = null
            setPadding(0, 0, 0, 0)
        }
        liveNoteEditLayout?.addView(liveNoteEditText)
        liveNoteBox.addView(liveNoteEditLayout)

        cardLayout.addView(liveNoteBox)
        rootLayout.addView(cardLayout)

        // ── 6. Smooth Touch Drag Handling & Interaction Wakeup ──
        var initialX = 0
        var initialY = 0
        var initialTouchX = 0f
        var initialTouchY = 0f

        cardLayout.setOnTouchListener { _, event ->
            recordInteraction()
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
            subtitleTextView?.text = subtitle
            liveNoteText = if (note.isNotBlank()) note else "No notes recorded yet."
            liveNoteTextView?.text = liveNoteText
            stopwatchSeconds = seconds
            isStopwatchRunning = running
            pulsingDot?.visibility = if (running) View.VISIBLE else View.GONE
            virtualScrollBadge?.visibility = if (liveNoteText.length > 10240) View.VISIBLE else View.GONE
            badgeDeadlineTextView?.text = deadlineTime
            badgeSubTaskTextView?.text = subTaskTime
            updateStopwatchDisplay()
        }
    }

    private fun updateStopwatchDisplay() {
        val h = stopwatchSeconds / 3600
        val m = (stopwatchSeconds % 3600) / 60
        val s = stopwatchSeconds % 60
        val str = if (h > 0) {
            String.format("%02d:%02d:%02d", h, m, s)
        } else {
            String.format("%02d:%02d", m, s)
        }
        badgeStopwatchTextView?.text = str
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
        mainHandler.removeCallbacks(inactivityRunnable)
        if (floatingView != null) {
            try {
                windowManager?.removeView(floatingView)
            } catch (e: Exception) {
                e.printStackTrace()
            }
            floatingView = null
        }
        super.onDestroy()
    }
}
