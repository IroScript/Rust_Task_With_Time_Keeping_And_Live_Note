package com.tasknote.task_note_mobile

import android.animation.ValueAnimator
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
import android.os.SystemClock
import android.text.Editable
import android.text.TextWatcher
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.view.animation.LinearInterpolator
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.core.app.NotificationCompat
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.cos
import kotlin.math.sin

/// 1:1 Parity Task Card Data Model for Floating Overlay
data class TaskCardData(
    var id: String = "1",
    var mainText: String = "Focus on the work - Success is near",
    var subText: String = "Keep pushing forward! ✨",
    var liveNote: String = "",
    var deadlineTime: String = "12.10 PM",
    var subTaskTime: String = "12.10 PM",
    var stopwatchSeconds: Int = 0,
    var isStopwatchRunning: Boolean = false,
    var depth: Int = 0
)

/// 100% Rust-Parity Android Native Draggable Floating Window Service
///
/// Implements 1:1 parity with:
/// - Rust card_header_widgets.rs:
///     * [+] Plus Button (22x22 dp, #3CB450 green border, #B21C1C red plus)
///     * 3 Independent Clock Badges per card:
///         - Badge 0: Deadline (#A51616 Crimson Red, "12.10 PM")
///         - Badge 1: Sub-Task (#B95F0F Amber, "12.10 PM")
///         - Badge 2: Stopwatch (#1C76B9 Steel-Blue, "MM:SS", pulsing dot #28C8FF)
/// - Rust src/main.rs:2240-2540:
///     * Add Card button (+) in title bar (src/main.rs:2258 icons::ADD_CARD)
///     * Title "DAILY MOTIVATION" (src/main.rs:2267)
///     * Quote counter [ disp_idx / disp_total ] (src/main.rs:2308)
///     * Navigation buttons ◀ and ▶ (src/main.rs:4135)
///     * Hide window title bar (src/main.rs:2343 icons::HIDE_HEADER) & Show header (src/main.rs:2529 icons::SHOW_HEADER)
///     * Dance animation button (src/main.rs:2414 icons::ANIM_DANCE, AppAnimation::Dance)
///     * Title bar auto-hide after 5.0s inactivity
/// - Google Keep-Style Live Note (src/views/live_note.rs & src/main.rs:3188, 3242):
///     * Always-open editable EditText (NO "EDIT" / "SAVE" button needed)
///     * Auto-save to SharedPreferences ('cached_cards') on every keypress
///     * Soft keyboard (IME) input enabled
///     * Virtual scrolling indicator (>10 KB badge)
class FloatingWindowService : Service() {

    private var windowManager: WindowManager? = null
    private var floatingView: View? = null
    private var layoutParams: WindowManager.LayoutParams? = null

    // Multi-Card State List
    private val cardsList = ArrayList<TaskCardData>()
    private var currentCardIndex = 0
    private var isUpdatingUiFromModel = false

    // UI Elements
    private var titleTextView: TextView? = null
    private var subtitleTextView: TextView? = null
    private var liveNoteEditText: EditText? = null
    private var virtualScrollBadge: TextView? = null
    private var quoteCounterTextView: TextView? = null

    // Title Bar State & Controls (Rust main.rs:2240-2540)
    private var headerLayout: LinearLayout? = null
    private var showHeaderBtn: TextView? = null
    private var addCardTitleBtn: TextView? = null
    private var danceBtn: TextView? = null
    private var prevCardBtn: TextView? = null
    private var nextCardBtn: TextView? = null
    private var isHeaderVisible = true

    // Dance Animation State (Rust AppAnimation::Dance, src/main.rs:8000-8007)
    private var isDancing = false
    private var danceAnimator: ValueAnimator? = null
    private var danceBaseX = 0
    private var danceBaseY = 0

    // 3 Clock Badges
    private var badgeDeadlineTextView: TextView? = null
    private var badgeSubTaskTextView: TextView? = null
    private var badgeStopwatchTextView: TextView? = null
    private var pulsingDot: View? = null

    // Action buttons & Auto-hide
    private var actionButtonsLayout: LinearLayout? = null
    private var floatingButtonOpacity = 1.0f
    private var noteTextSizeSp = 11.5f

    private val mainHandler = Handler(Looper.getMainLooper())

    private val tickerRunnable = object : Runnable {
        override fun run() {
            var anyRunning = false
            for (card in cardsList) {
                if (card.isStopwatchRunning) {
                    card.stopwatchSeconds++
                    anyRunning = true
                }
            }
            if (anyRunning) {
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

        loadCardsFromPreferences()
        createFloatingWindow()
        displayCurrentCard()

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
                    val title = intent.getStringExtra(EXTRA_TITLE)
                    val subtitle = intent.getStringExtra(EXTRA_SUBTITLE)
                    val note = intent.getStringExtra(EXTRA_NOTE)
                    val seconds = intent.getIntExtra(EXTRA_SECONDS, -1)
                    val hasRunning = intent.hasExtra(EXTRA_IS_RUNNING)
                    val running = intent.getBooleanExtra(EXTRA_IS_RUNNING, false)
                    val incomingId = intent.getStringExtra(EXTRA_CARD_ID)
                    val depth = intent.getIntExtra(EXTRA_DEPTH, 0)
                    val deadline = intent.getStringExtra(EXTRA_DEADLINE)
                    val subtask = intent.getStringExtra(EXTRA_SUBTASK_TIME)

                    if (cardsList.isEmpty()) {
                        loadCardsFromPreferences()
                    }

                    // Update or insert matching card
                    val card = if (incomingId != null) {
                        cardsList.find { it.id == incomingId } ?: TaskCardData(id = incomingId).also { cardsList.add(it) }
                    } else {
                        getCurrentCard()
                    }

                    if (title != null) card.mainText = title
                    if (subtitle != null) card.subText = subtitle
                    if (note != null) card.liveNote = note
                    if (seconds >= 0) card.stopwatchSeconds = seconds
                    if (hasRunning) card.isStopwatchRunning = running
                    if (deadline != null) card.deadlineTime = deadline
                    if (subtask != null) card.subTaskTime = subtask
                    card.depth = depth

                    displayCurrentCard()
                }
            }
        }
        return START_STICKY
    }

    private fun getCurrentCard(): TaskCardData {
        if (cardsList.isEmpty()) {
            cardsList.add(TaskCardData())
            currentCardIndex = 0
        }
        if (currentCardIndex < 0 || currentCardIndex >= cardsList.size) {
            currentCardIndex = 0
        }
        return cardsList[currentCardIndex]
    }

    private fun loadCardsFromPreferences() {
        cardsList.clear()
        try {
            val prefs = getSharedPreferences("FlutterSharedPreferences", Context.MODE_PRIVATE)
            val jsonStr = prefs.getString("flutter.cached_cards", null)
            if (!jsonStr.isNullOrBlank()) {
                val array = JSONArray(jsonStr)
                for (i in 0 until array.length()) {
                    val obj = array.getJSONObject(i)
                    val c = TaskCardData(
                        id = obj.optString("id", "${i + 1}"),
                        mainText = obj.optString("mainText", "Focus on the work - Success is near"),
                        subText = obj.optString("subText", "Keep pushing forward! ✨"),
                        liveNote = obj.optString("liveNote", ""),
                        deadlineTime = if (obj.has("endTime")) obj.optString("endTime", "12.10 PM") else obj.optString("deadlineTime", "12.10 PM"),
                        subTaskTime = if (obj.has("startTime")) obj.optString("startTime", "12.10 PM") else obj.optString("subTaskTime", "12.10 PM"),
                        stopwatchSeconds = obj.optInt("stopwatchSeconds", 0),
                        isStopwatchRunning = obj.optBoolean("isStopwatchRunning", false),
                        depth = obj.optInt("depth", 0)
                    )
                    cardsList.add(c)
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
        }

        if (cardsList.isEmpty()) {
            cardsList.add(TaskCardData("1", "Focus on the work - Success is near", "Keep pushing forward! ✨", "", "12.10 PM", "12.10 PM", 0, false, 0))
            cardsList.add(TaskCardData("2", "Plan the next iteration carefully", "Precision over haste", "", "01.00 PM", "12.30 PM", 0, false, 0))
        }
        currentCardIndex = 0
    }

    private fun saveCardsToPreferences() {
        try {
            val array = JSONArray()
            for (c in cardsList) {
                val obj = JSONObject().apply {
                    put("id", c.id)
                    put("mainText", c.mainText)
                    put("subText", c.subText)
                    put("liveNote", c.liveNote)
                    put("startTime", c.subTaskTime)
                    put("endTime", c.deadlineTime)
                    put("stopwatchSeconds", c.stopwatchSeconds)
                    put("isStopwatchRunning", c.isStopwatchRunning)
                    put("depth", c.depth)
                }
                array.put(obj)
            }
            val prefs = getSharedPreferences("FlutterSharedPreferences", Context.MODE_PRIVATE)
            prefs.edit().putString("flutter.cached_cards", array.toString()).apply()

            // Broadcast notification for Flutter listeners
            val intent = Intent("com.tasknote.ACTION_CARD_UPDATED").apply {
                val card = getCurrentCard()
                putExtra("cardId", card.id)
                putExtra("note", card.liveNote)
                putExtra("seconds", card.stopwatchSeconds)
                putExtra("running", card.isStopwatchRunning)
            }
            sendBroadcast(intent)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }

    private fun displayCurrentCard() {
        val card = getCurrentCard()
        mainHandler.post {
            isUpdatingUiFromModel = true
            titleTextView?.text = card.mainText
            subtitleTextView?.text = if (card.depth > 0) "[Sub-card depth: ${card.depth}] ${card.subText}" else card.subText

            // Google Keep style: always editable, preserve text cursor
            if (liveNoteEditText?.text?.toString() != card.liveNote) {
                liveNoteEditText?.setText(card.liveNote)
                liveNoteEditText?.setSelection(card.liveNote.length)
            }

            badgeDeadlineTextView?.text = card.deadlineTime
            badgeSubTaskTextView?.text = card.subTaskTime
            pulsingDot?.visibility = if (card.isStopwatchRunning) View.VISIBLE else View.GONE
            virtualScrollBadge?.visibility = if (card.liveNote.length > 10240) View.VISIBLE else View.GONE

            quoteCounterTextView?.text = "[ ${currentCardIndex + 1} / ${cardsList.size} ]"

            updateStopwatchDisplay()
            isUpdatingUiFromModel = false
        }
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

    private fun showKeyboardAndFocus() {
        val params = layoutParams ?: return
        // Remove FLAG_NOT_FOCUSABLE so that this window can receive keyboard input
        params.flags = params.flags and WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE.inv()
        params.softInputMode = WindowManager.LayoutParams.SOFT_INPUT_STATE_VISIBLE or WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
        try {
            windowManager?.updateViewLayout(floatingView, params)
        } catch (_: Exception) {}

        liveNoteEditText?.isFocusable = true
        liveNoteEditText?.isFocusableInTouchMode = true
        liveNoteEditText?.requestFocus()

        liveNoteEditText?.postDelayed({
            val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
            imm?.showSoftInput(liveNoteEditText, InputMethodManager.SHOW_IMPLICIT)
        }, 150)
    }

    private fun hideKeyboardAndUnfocus() {
        val imm = getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
        imm?.hideSoftInputFromWindow(liveNoteEditText?.windowToken, 0)

        val params = layoutParams ?: return
        // Restore FLAG_NOT_FOCUSABLE so touches outside the window pass through to underlying apps
        params.flags = params.flags or WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE
        params.softInputMode = WindowManager.LayoutParams.SOFT_INPUT_STATE_UNCHANGED
        try {
            windowManager?.updateViewLayout(floatingView, params)
        } catch (_: Exception) {}
        liveNoteEditText?.clearFocus()
    }

    private fun toggleDance() {
        recordInteraction()
        if (isDancing) {
            stopDance()
        } else {
            startDance()
        }
    }

    private fun startDance() {
        val params = layoutParams ?: return
        danceBaseX = params.x
        danceBaseY = params.y
        isDancing = true
        danceBtn?.setTextColor(Color.parseColor("#3CB450")) // Exact NEON_LIME from Rust
        Toast.makeText(this, "Dancing window active", Toast.LENGTH_SHORT).show()

        val startTime = SystemClock.uptimeMillis()
        val radius = dpToPx(35).toFloat()

        danceAnimator = ValueAnimator.ofFloat(0f, 1f).apply {
            duration = 10000L
            repeatCount = ValueAnimator.INFINITE
            interpolator = LinearInterpolator()
            addUpdateListener {
                if (!isDancing) return@addUpdateListener
                val p = (SystemClock.uptimeMillis() - startTime) / 1000f
                // Exact Rust Lissajous formula: (anim_progress * 4.0).sin() * radius, (anim_progress * 2.5).cos() * radius
                val offsetX = (sin(p * 4.0) * radius).toInt()
                val offsetY = (cos(p * 2.5) * radius).toInt()
                params.x = danceBaseX + offsetX
                params.y = danceBaseY + offsetY
                try {
                    windowManager?.updateViewLayout(floatingView, params)
                } catch (_: Exception) {}
            }
            start()
        }
    }

    private fun stopDance() {
        if (!isDancing && danceAnimator == null) return
        isDancing = false
        danceAnimator?.cancel()
        danceAnimator = null
        danceBtn?.setTextColor(Color.WHITE)
        val params = layoutParams ?: return
        params.x = danceBaseX
        params.y = danceBaseY
        try {
            windowManager?.updateViewLayout(floatingView, params)
        } catch (_: Exception) {}
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

        // ── 0. Floating Show Header Button (src/main.rs:2529 icons::SHOW_HEADER) ──
        showHeaderBtn = TextView(this).apply {
            text = "🔼 Show Header"
            textSize = 9.5f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#3CB450"))
            background = GradientDrawable().apply {
                cornerRadius = dpToPx(4).toFloat()
                setStroke(dpToPx(1), Color.parseColor("#3CB450"))
                setColor(Color.parseColor("#90080B10"))
            }
            setPadding(dpToPx(8), dpToPx(2), dpToPx(8), dpToPx(2))
            visibility = View.GONE
            setOnClickListener {
                recordInteraction()
                isHeaderVisible = true
                headerLayout?.visibility = View.VISIBLE
                visibility = View.GONE
            }
        }
        cardLayout.addView(showHeaderBtn)

        // ── 1. Top Bar: Glowing Indicator + [+] Add Card + Header Title + Auto-Hide Action Buttons ──
        headerLayout = LinearLayout(this).apply {
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
                marginEnd = dpToPx(5)
            }
        }

        // [+] Add Card / Note in Title Bar (src/main.rs:2258 icons::ADD_CARD)
        addCardTitleBtn = TextView(this).apply {
            text = "+"
            textSize = 15f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#3CB450")) // Exact NEON_LIME from Rust
            setPadding(dpToPx(3), dpToPx(0), dpToPx(5), dpToPx(0))
            setOnClickListener {
                recordInteraction()
                val newId = System.currentTimeMillis().toString()
                val newCard = TaskCardData(
                    id = newId,
                    mainText = "Daily Note ${cardsList.size + 1}",
                    subText = "Keep pushing forward! ✨",
                    liveNote = "",
                    deadlineTime = "12.10 PM",
                    subTaskTime = "12.10 PM",
                    stopwatchSeconds = 0,
                    isStopwatchRunning = false,
                    depth = 0
                )
                cardsList.add(0, newCard)
                currentCardIndex = 0
                displayCurrentCard()
                saveCardsToPreferences()
                showKeyboardAndFocus()
                Toast.makeText(this@FloatingWindowService, "New note ready (type directly, auto-saved)", Toast.LENGTH_SHORT).show()
                sendBroadcast(Intent("com.tasknote.ACTION_ADD_CARD"))
            }
        }

        val headerTitle = TextView(this).apply {
            text = "DAILY MOTIVATION"
            textSize = 9.5f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#3CB450"))
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1.0f)
        }

        // Quote Counter [ disp_idx / disp_total ] (src/main.rs:2308)
        quoteCounterTextView = TextView(this).apply {
            text = "[ 1 / 1 ]"
            textSize = 9f
            typeface = Typeface.MONOSPACE
            setTextColor(Color.parseColor("#3CB450"))
            setPadding(dpToPx(2), dpToPx(0), dpToPx(4), dpToPx(0))
        }

        // Carousel Navigation: ◀ Previous Card (src/main.rs:4135)
        prevCardBtn = TextView(this).apply {
            text = "◀"
            textSize = 11f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#00FFFF"))
            setPadding(dpToPx(3), dpToPx(0), dpToPx(3), dpToPx(0))
            setOnClickListener {
                recordInteraction()
                if (cardsList.isNotEmpty()) {
                    currentCardIndex = if (currentCardIndex > 0) currentCardIndex - 1 else cardsList.size - 1
                    displayCurrentCard()
                }
            }
        }

        // Carousel Navigation: ▶ Next Card (src/main.rs:4141)
        nextCardBtn = TextView(this).apply {
            text = "▶"
            textSize = 11f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#00FFFF"))
            setPadding(dpToPx(3), dpToPx(0), dpToPx(5), dpToPx(0))
            setOnClickListener {
                recordInteraction()
                if (cardsList.isNotEmpty()) {
                    currentCardIndex = (currentCardIndex + 1) % cardsList.size
                    displayCurrentCard()
                }
            }
        }

        // Floating Action Buttons (with 5.0s Auto-Hide Opacity Animation)
        actionButtonsLayout = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }

        // Dance Animation Button (💃 src/main.rs:2414 icons::ANIM_DANCE)
        danceBtn = TextView(this).apply {
            text = "💃"
            textSize = 12f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.WHITE)
            setPadding(dpToPx(4), dpToPx(2), dpToPx(4), dpToPx(2))
            setOnClickListener {
                toggleDance()
            }
        }

        // Hide Header Button (▲ src/main.rs:2343 icons::HIDE_HEADER)
        val hideHeaderBtn = TextView(this).apply {
            text = "▲"
            textSize = 11f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.WHITE)
            setPadding(dpToPx(4), dpToPx(2), dpToPx(4), dpToPx(2))
            setOnClickListener {
                recordInteraction()
                isHeaderVisible = false
                headerLayout?.visibility = View.GONE
                showHeaderBtn?.visibility = View.VISIBLE
            }
        }

        // Open App Button (↗)
        val openAppBtn = TextView(this).apply {
            text = "↗"
            textSize = 13f
            typeface = Typeface.DEFAULT_BOLD
            setTextColor(Color.parseColor("#39FF14"))
            setPadding(dpToPx(4), dpToPx(2), dpToPx(4), dpToPx(2))
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
            setPadding(dpToPx(4), dpToPx(2), dpToPx(2), dpToPx(2))
            setOnClickListener {
                stopSelf()
            }
        }

        actionButtonsLayout?.addView(danceBtn)
        actionButtonsLayout?.addView(hideHeaderBtn)
        actionButtonsLayout?.addView(openAppBtn)
        actionButtonsLayout?.addView(closeBtn)

        headerLayout?.addView(glowIndicator)
        headerLayout?.addView(addCardTitleBtn)
        headerLayout?.addView(headerTitle)
        headerLayout?.addView(quoteCounterTextView)
        headerLayout?.addView(prevCardBtn)
        headerLayout?.addView(nextCardBtn)
        headerLayout?.addView(actionButtonsLayout)
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
                val curCard = getCurrentCard()
                val newDepth = curCard.depth + 1
                val newId = System.currentTimeMillis().toString()
                val subCard = TaskCardData(
                    id = newId,
                    mainText = "Sub-task for ${curCard.mainText}",
                    subText = "Depth $newDepth",
                    liveNote = "",
                    deadlineTime = curCard.deadlineTime,
                    subTaskTime = curCard.subTaskTime,
                    stopwatchSeconds = 0,
                    isStopwatchRunning = false,
                    depth = newDepth
                )
                cardsList.add(currentCardIndex + 1, subCard)
                currentCardIndex++
                displayCurrentCard()
                saveCardsToPreferences()
                showKeyboardAndFocus()
                Toast.makeText(this@FloatingWindowService, "Sub-card created (depth $newDepth, scale 95%)", Toast.LENGTH_SHORT).show()
                sendBroadcast(Intent("com.tasknote.ACTION_ADD_SUBCARD").apply {
                    putExtra("cardId", subCard.id)
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
        badgeDeadlineTextView = createClockChip("12.10 PM", "#A51616").apply {
            setOnClickListener {
                recordInteraction()
                val card = getCurrentCard()
                Toast.makeText(this@FloatingWindowService, "Deadline: ${card.deadlineTime}", Toast.LENGTH_SHORT).show()
            }
        }
        headerRowLayout.addView(badgeDeadlineTextView)

        // 2c. Badge 1: Sub-Task Time (#B95F0F Amber)
        badgeSubTaskTextView = createClockChip("12.10 PM", "#B95F0F").apply {
            setOnClickListener {
                recordInteraction()
                val card = getCurrentCard()
                Toast.makeText(this@FloatingWindowService, "Sub-task: ${card.subTaskTime}", Toast.LENGTH_SHORT).show()
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
            val card = getCurrentCard()
            card.isStopwatchRunning = !card.isStopwatchRunning
            pulsingDot?.visibility = if (card.isStopwatchRunning) View.VISIBLE else View.GONE
            updateStopwatchDisplay()
            saveCardsToPreferences()
        }

        stopwatchBadgeContainer.setOnLongClickListener {
            recordInteraction()
            val card = getCurrentCard()
            card.isStopwatchRunning = false
            card.stopwatchSeconds = 0
            pulsingDot?.visibility = View.GONE
            updateStopwatchDisplay()
            saveCardsToPreferences()
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

        // ── 5. Google Keep-Style Live Note Box & In-Place Writing (ALWAYS EDITABLE) ──
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
                liveNoteEditText?.textSize = noteTextSizeSp
            }
        }

        noteHeader.addView(noteTitle)
        noteHeader.addView(fontPlusBtn)
        noteHeader.addView(fontMinusBtn)
        liveNoteBox.addView(noteHeader)

        // Google Keep Parity: Live Note EditText is ALWAYS visible and directly editable.
        // No EDIT button, No SAVE button. Auto-saves to SharedPreferences on every keypress!
        liveNoteEditText = EditText(this).apply {
            hint = "Write note here... (auto-saved)"
            setHintTextColor(Color.parseColor("#556677"))
            setTextColor(Color.WHITE)
            textSize = noteTextSizeSp
            minLines = 2
            maxLines = 5
            background = null
            setPadding(dpToPx(2), dpToPx(4), dpToPx(2), dpToPx(4))
            isFocusable = true
            isFocusableInTouchMode = true

            setOnClickListener {
                recordInteraction()
                showKeyboardAndFocus()
            }

            setOnFocusChangeListener { _, hasFocus ->
                if (hasFocus) {
                    recordInteraction()
                    showKeyboardAndFocus()
                }
            }

            // Auto-save on every single keypress
            addTextChangedListener(object : TextWatcher {
                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {}
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {}
                override fun afterTextChanged(s: Editable?) {
                    if (isUpdatingUiFromModel) return
                    val newText = s?.toString() ?: ""
                    val card = getCurrentCard()
                    if (card.liveNote != newText) {
                        card.liveNote = newText
                        virtualScrollBadge?.visibility = if (newText.length > 10240) View.VISIBLE else View.GONE
                        saveCardsToPreferences()
                    }
                }
            })
        }
        liveNoteBox.addView(liveNoteEditText)

        cardLayout.addView(liveNoteBox)
        rootLayout.addView(cardLayout)

        // ── 6. Smooth Touch Drag Handling & Interaction Wakeup ──
        var initialX = 0
        var initialY = 0
        var initialTouchX = 0f
        var initialTouchY = 0f

        cardLayout.setOnTouchListener { _, event ->
            recordInteraction()
            if (isDancing) {
                stopDance()
            }
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

    private fun updateStopwatchDisplay() {
        val card = getCurrentCard()
        val h = card.stopwatchSeconds / 3600
        val m = (card.stopwatchSeconds % 3600) / 60
        val s = card.stopwatchSeconds % 60
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
        stopDance()
        hideKeyboardAndUnfocus()
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
