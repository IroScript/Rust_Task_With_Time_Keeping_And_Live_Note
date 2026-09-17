# AGY APK BUILD VERIFICATION MANDATE & RESOLUTION PROTOCOL

> 🛑 **MANDATORY SYSTEM DIRECTIVE FROM USER (ইরাক ভাইয়া)** 🛑
> 
> **"AGY-এর কাজ শেষ হবে না যতক্ষণ না পর্যন্ত APK বিল্ডিং ইস্যু ১০০% সমাধান এবং যাচাই (Verify) সম্পন্ন হচ্ছে।"**
> 
> *AGY's work is STRICTLY INCOMPLETE until the Android APK build process is 100% resolved, verified, and a working release APK artifact is successfully generated and confirmed.*

---

## ১. বর্তমান অবস্থা ও মূল সমস্যার বিশ্লেষণ (Root Cause Analysis)

পূর্ববর্তী রানসমূহে APK বিল্ড ব্যর্থ হওয়ার মূল কারণগুলো বিস্তারিতভাবে চিহ্নিত করা হয়েছে:

1. **Gradle JVM Memory Limit Exceeded:**
   - `mobile/android/gradle.properties`-এ `-Xmx8G -XX:MaxMetaspaceSize=4G` কনফিগার করা ছিল।
   - GitHub Actions `ubuntu-latest` রানারে মোট RAM থাকে মাত্র ৭ জিবি। ফলে ১২ জিবি রিকোয়েস্টের কারণে সাথে সাথে JVM OOM ক্র্যাশ করত।
   - **সমাধান:** `-Xmx3072m -XX:MaxMetaspaceSize=1024m` সেট করা হয়েছে।

2. **Android Studio Bundled Java 17 বনাম Java 21 (AGP 9.1 Conflict):**
   - Flutter 3.47 এ ব্যবহৃত Android Gradle Plugin (AGP) 9.1 এবং Gradle 9.3 এর জন্য বাধ্যতামূলক Java 21 প্রয়োজন।
   - Flutter ডিফল্টভাবে `JAVA_HOME` চেক করার আগে Android Studio-এর বান্ডেলড Java 17 কে অগ্রাধিকার দেয়। ফলে রানারে Java 21 থাকা সত্ত্বেও Flutter Android Studio-এর Java 17 ব্যবহার করে ফেইল করছিল।
   - **সমাধান:** ওয়ার্কফ্লোতে স্পষ্টভাবে `flutter config --jdk-dir "$JAVA_HOME"` যুক্ত করে রানারের Java 21 ব্যবহার নিশ্চিত করা হয়েছে।

3. **Android SDK Platform 36 বনাম Runner Pre-installed Platform 35:**
   - Flutter 3.47 ডিফল্টভাবে `compileSdkVersion 36` ও `ndkVersion 28.2.13676358` খুঁজছিল, যা রানারে ইনস্টল করা ছিল না।
   - **সমাধান:** `mobile/android/app/build.gradle.kts`-এ `compileSdk = 35`, `minSdk = 21`, `targetSdk = 35` নির্ধারণ করা হয়েছে এবং অপ্রয়োজনীয় NDK ডিপেন্ডেন্সি বাদ দেওয়া হয়েছে যাতে রানারের প্রি-ইনস্টল্ড অ্যান্ড্রয়েড ৩৫ এসডিকে সরাসরি ব্যবহার হতে পারে।

4. **স্বচ্ছ লগিং ও আর্টফ্যাক্ট ট্র্যাকিং:**
   - `flutter build apk --release --verbose 2>&1 | tee ../build.log` এবং `if: always()` দিয়ে `build.log` আর্টফ্যাক্ট আপলোড কনফিগার করা হয়েছে।

---

## ২. অবিরাম যাচাইকরণ চেকলিস্ট (Continuous Verification Checklist)

AGY নিম্নলিখিত প্রতিটি চেকলিস্ট আইটেম প্রমাণিত না হওয়া পর্যন্ত এই টাস্ককে সম্পন্ন ঘোষণা করতে পারবে না:

- [x] ১. Gradle JVM মেমোরি আর্গুমেন্ট ৩ জিবিতে অপ্টিমাইজ করা।
- [x] ২. `compileSdk = 35` এবং `targetSdk = 35` ফিক্স কার্যকর করা।
- [x] ৩. `flutter config --jdk-dir "$JAVA_HOME"` দিয়ে Java 21 প্রয়োগ নিশ্চিত করা।
- [x] ৪. `build.log` আর্টফ্যাক্ট ট্র্যাকিং ওয়ার্কফ্লোতে যুক্ত করা।
- [ ] ৫. GitHub-এ কোড পুশ করে নতুন GitHub Actions রান শুরু করা।
- [ ] ৬. GitHub Actions-এ `Build Android Release APK` স্টেপ সফল (Success) হিসেবে ভেরিফাই করা।
- [ ] ৭. `task-note-cyberpunk-release-apk` আর্টফ্যাক্ট ডাউনলোড লিংক নিশ্চিত করা।
- [ ] ৮. ইরাক ভাইয়াকে চূড়ান্ত ভেরিফিকেশন রিপোর্ট ও ডাউনলোড লিংক হস্তান্তর করা।

---

## ৩. এজিওয়াই সিস্টেমের অঙ্গীকার (AGY Commitment)

AGY প্রতিজ্ঞাবদ্ধ যে, যতক্ষণ না পর্যন্ত GitHub Actions সফলভাবে রিলিজ APK কম্পাইল করে ডাউনলোড লিংক তৈরি করছে এবং তা কঠোর ভেরিফিকেশন প্রোটোকল দিয়ে প্রমাণিত হচ্ছে, ততক্ষণ AGY এই ইস্যুর ওপর সক্রিয় থেকে সমাধান ও পুনরাবৃত্তি যাচাই চালিয়ে যাবে।
