# AGY APK BUILD VERIFICATION MANDATE & RESOLUTION PROTOCOL

> 🛑 **MANDATORY SYSTEM DIRECTIVE FROM USER (ইরাক ভাইয়া)** 🛑
> 
> **"AGY-এর কাজ শেষ হবে না যতক্ষণ না পর্যন্ত APK বিল্ডিং ইস্যু ১০০% সমাধান এবং যাচাই (Verify) সম্পন্ন হচ্ছে।"**
> 
> *AGY's work is STRICTLY INCOMPLETE until the Android APK build process is 100% resolved, verified, and a working release APK artifact is successfully generated and confirmed.*

---

## ১. বর্তমান অবস্থা ও মূল সমস্যার সমাধান (Status & Solved Milestones)

পূর্ববর্তী রানসমূহে APK বিল্ডের প্রতিটি পর্যায় সফলভাবে সমাধান করা হয়েছে:

1. **Gradle JVM Memory Limit Exceeded:**
   - `mobile/android/gradle.properties`-এ `-Xmx3072m -XX:MaxMetaspaceSize=1024m` সেট করা হয়েছে।
   - **স্ট্যাটাস:** [x] সফলভাবে কাজ করেছে (কোনো OOM ক্র্যাশ ঘটেনি)।

2. **Android Studio Bundled Java 17 বনাম Java 21 (AGP 9.1 Conflict):**
   - `flutter config --jdk-dir "$JAVA_HOME"` যুক্ত করে রানারের Java 21 প্রয়োগ করা হয়েছে।
   - **স্ট্যাটাস:** [x] সফলভাবে সমাধান হয়েছে।

3. **Android SDK Platform 36 বনাম Runner Pre-installed Platform 35:**
   - `compileSdk = 35`, `minSdk = 21`, `targetSdk = 35` নির্ধারণ করা হয়েছে।
   - **স্ট্যাটাস:** [x] সফলভাবে সমাধান হয়েছে।

4. **APK কম্পাইলেশন স্ট্যাটাস (Run ID: 35190778444, Step 9):**
   - **`Build Android Release APK`**: `completed` -> `conclusion: success`!
   - **প্রমাণ:** Gradle রিলিজ APK ১০০% সফলভাবে কম্পাইল করেছে (এক্সিট কোড ০)।

5. **আর্টফ্যাক্ট আপলোড পাথ অপ্টিমাইজেশন (Final Mile):**
   - আধুনিক AGP APK ফাইলকে `mobile/build/app/outputs/apk/release/` অথবা `mobile/build/app/outputs/flutter-apk/` ডিরেক্টরিতে রাখতে পারে।
   - **সমাধান:** ওয়ার্কফ্লোতে `path: mobile/build/**/*.apk` গ্লোব প্যাটার্ন এবং `find mobile/build -name "*.apk" -ls` যুক্ত করা হয়েছে যাতে উৎপাদিত যেকোনো APK সরাসরি আর্টফ্যাক্টে আপলোড হয়।

---

## ২. অবিরাম যাচাইকরণ চেকলিস্ট (Continuous Verification Checklist)

- [x] ১. Gradle JVM মেমোরি আর্গুমেন্ট ৩ জিবিতে অপ্টিমাইজ করা।
- [x] ২. `compileSdk = 35` এবং `targetSdk = 35` ফিক্স কার্যকর করা।
- [x] ৩. `flutter config --jdk-dir "$JAVA_HOME"` দিয়ে Java 21 প্রয়োগ নিশ্চিত করা।
- [x] ৪. `build.log` আর্টফ্যাক্ট ট্র্যাকিং কার্যকর করা (সফলভাবে ১০৭ KB লগ জেনারেট হয়েছে)।
- [x] ৫. **`Build Android Release APK` স্টেপ সফল (Success) হিসেবে রান ৩৫১৯০৭৭৮৪৪৪-এ প্রমাণিত!**
- [x] ৬. রিলিজ APK আর্টফ্যাক্ট পাথ `mobile/build/**/*.apk` গ্লোবে রূপান্তর।
- [ ] ৭. কোড পুশ করে আর্টফ্যাক্ট ডাউনলোড লিংক নিশ্চিত করা।
- [ ] ৮. ইরাক ভাইয়াকে চূড়ান্ত ভেরিফিকেশন রিপোর্ট ও ডাউনলোড লিংক হস্তান্তর করা।

---

## ৩. এজিওয়াই সিস্টেমের অঙ্গীকার (AGY Commitment)

AGY প্রতিজ্ঞাবদ্ধ যে, যতক্ষণ না পর্যন্ত GitHub Actions সফলভাবে রিলিজ APK কম্পাইল করে আর্টফ্যাক্ট ডাউনলোড লিংক প্রদান করছে, ততক্ষণ AGY এই ইস্যুর ওপর সক্রিয় থেকে সমাধান ও পুনরাবৃত্তি যাচাই চালিয়ে যাবে।
