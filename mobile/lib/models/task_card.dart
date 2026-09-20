enum ClockMode {
  taskDeadline,
  subTaskTime,
  stopwatch,
}

extension ClockModeExtension on ClockMode {
  String get label {
    switch (this) {
      case ClockMode.taskDeadline:
        return 'Deadline';
      case ClockMode.subTaskTime:
        return 'Sub-task';
      case ClockMode.stopwatch:
        return 'Stopwatch';
    }
  }

  ClockMode get next {
    switch (this) {
      case ClockMode.taskDeadline:
        return ClockMode.subTaskTime;
      case ClockMode.subTaskTime:
        return ClockMode.stopwatch;
      case ClockMode.stopwatch:
        return ClockMode.taskDeadline;
    }
  }
}

/// 1:1 Parity with Rust Quote struct in src/main.rs:602
class TaskCard {
  String id;
  String mainText;
  String subText;
  String startTime;
  String endTime;
  int stopwatchSeconds;
  bool isStopwatchRunning;
  ClockMode clockMode;
  bool isHidden;
  int depth;
  String? parentId;
  String liveNote;
  DateTime createdAt;
  DateTime updatedAt;
  int totalLines;

  // Rust Quote styling overrides
  double? mainTextSize;
  double? subTextSize;
  int? mainTextColor;
  int? subTextColor;
  double? mainLineGap;
  double? subLineGap;
  double? betweenGap;
  int? intervalSecs;

  // Schedule information
  String? scheduledDate;
  String? scheduledTime;
  int orderIndex;

  TaskCard({
    required this.id,
    required this.mainText,
    this.subText = "Keep pushing - You're doing great! ✨",
    this.startTime = '12:10 PM',
    this.endTime = '12:10 PM',
    this.stopwatchSeconds = 0,
    this.isStopwatchRunning = false,
    this.clockMode = ClockMode.stopwatch,
    this.isHidden = false,
    this.depth = 0,
    this.parentId,
    this.liveNote = '',
    DateTime? createdAt,
    DateTime? updatedAt,
    this.totalLines = 0,
    this.mainTextSize,
    this.subTextSize,
    this.mainTextColor,
    this.subTextColor,
    this.mainLineGap,
    this.subLineGap,
    this.betweenGap,
    this.intervalSecs,
    this.scheduledDate,
    this.scheduledTime,
    this.orderIndex = 0,
  })  : createdAt = createdAt ?? DateTime.now(),
        updatedAt = updatedAt ?? DateTime.now();

  String get formattedStopwatch {
    final minutes = (stopwatchSeconds ~/ 60).toString().padLeft(2, '0');
    final seconds = (stopwatchSeconds % 60).toString().padLeft(2, '0');
    return '$minutes:$seconds';
  }

  factory TaskCard.fromAxumJson(Map<String, dynamic> json) {
    return TaskCard(
      id: json['id']?.toString() ?? '',
      mainText: json['title'] ?? 'Untitled Card',
      subText: "Keep pushing - You're doing great! ✨",
      startTime: '12:10 PM',
      endTime: '12:10 PM',
      totalLines: json['total_lines'] is int ? json['total_lines'] : 0,
      createdAt: json['created_at'] != null
          ? DateTime.tryParse(json['created_at']) ?? DateTime.now()
          : DateTime.now(),
      updatedAt: json['updated_at'] != null
          ? DateTime.tryParse(json['updated_at']) ?? DateTime.now()
          : DateTime.now(),
    );
  }

  Map<String, dynamic> toAxumJson() {
    return {
      'title': mainText,
    };
  }

  Map<String, dynamic> toLocalJson() {
    return {
      'id': id,
      'mainText': mainText,
      'subText': subText,
      'startTime': startTime,
      'endTime': endTime,
      'stopwatchSeconds': stopwatchSeconds,
      'isStopwatchRunning': isStopwatchRunning,
      'clockMode': clockMode.index,
      'isHidden': isHidden,
      'depth': depth,
      'parentId': parentId,
      'liveNote': liveNote,
      'createdAt': createdAt.toIso8601String(),
      'updatedAt': updatedAt.toIso8601String(),
      'totalLines': totalLines,
      'mainTextSize': mainTextSize,
      'subTextSize': subTextSize,
      'mainTextColor': mainTextColor,
      'subTextColor': subTextColor,
      'mainLineGap': mainLineGap,
      'subLineGap': subLineGap,
      'betweenGap': betweenGap,
      'intervalSecs': intervalSecs,
      'scheduledDate': scheduledDate,
      'scheduledTime': scheduledTime,
      'orderIndex': orderIndex,
    };
  }

  factory TaskCard.fromLocalJson(Map<String, dynamic> json) {
    return TaskCard(
      id: json['id']?.toString() ?? '',
      mainText: json['mainText'] ?? '',
      subText: json['subText'] ?? '',
      startTime: json['startTime'] ?? '12:10 PM',
      endTime: json['endTime'] ?? '12:10 PM',
      stopwatchSeconds: json['stopwatchSeconds'] ?? 0,
      isStopwatchRunning: json['isStopwatchRunning'] ?? false,
      clockMode: ClockMode.values[(json['clockMode'] ?? 2) % ClockMode.values.length],
      isHidden: json['isHidden'] ?? false,
      depth: json['depth'] ?? 0,
      parentId: json['parentId'],
      liveNote: json['liveNote'] ?? '',
      createdAt: json['createdAt'] != null
          ? DateTime.tryParse(json['createdAt']) ?? DateTime.now()
          : DateTime.now(),
      updatedAt: json['updatedAt'] != null
          ? DateTime.tryParse(json['updatedAt']) ?? DateTime.now()
          : DateTime.now(),
      totalLines: json['totalLines'] ?? 0,
      mainTextSize: (json['mainTextSize'] as num?)?.toDouble(),
      subTextSize: (json['subTextSize'] as num?)?.toDouble(),
      mainTextColor: json['mainTextColor'] as int?,
      subTextColor: json['subTextColor'] as int?,
      mainLineGap: (json['mainLineGap'] as num?)?.toDouble(),
      subLineGap: (json['subLineGap'] as num?)?.toDouble(),
      betweenGap: (json['betweenGap'] as num?)?.toDouble(),
      intervalSecs: json['intervalSecs'] as int?,
      scheduledDate: json['scheduledDate'] as String?,
      scheduledTime: json['scheduledTime'] as String?,
      orderIndex: json['orderIndex'] ?? 0,
    );
  }
}
