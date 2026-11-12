# Keep JNI native bridge methods
-keepclasseswithmembers class com.diba.blocker.core.NativeBridge {
    native <methods>;
}

# Keep models serialized by serde
-keepclassmembers class * {
    @com.google.gson.annotations.SerializedName <fields>;
}
