package com.mora.rustandroid

import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView

class MainActivity : Activity() {
    private lateinit var result: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(32, 40, 32, 40)
            layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
        }
        val title = TextView(this).apply {
            text = "Mora Rust Android Core"
            textSize = 24f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
        }
        val description = TextView(this).apply {
            text = "Demo Kotlin ↔ Rust melalui JNI (API v2)."
            textSize = 15f
            gravity = Gravity.CENTER
            setPadding(0, 16, 0, 24)
        }
        result = TextView(this).apply {
            text = "Belum menjalankan pengujian."
            textSize = 15f
            gravity = Gravity.CENTER
            setPadding(0, 16, 0, 24)
        }
        val testButton = Button(this).apply {
            text = "Jalankan Semua Rust Tests"
            setOnClickListener { runNativeTest() }
        }
        root.addView(title)
        root.addView(description)
        root.addView(result)
        root.addView(testButton)
        setContentView(root)
    }

    private fun runNativeTest() {
        result.text = try {
            val version = NativeCore.nativeCoreApiVersion()
            val capabilities = NativeCore.nativeCoreCapabilities()
            val sum = NativeCore.nativeAdd(20, 22)
            val normalizedLabel = NativeCore.nativeNormalizeProfileLabel("  Daily Profile  ")
            val rejectsEmptyLabel = NativeCore.nativeNormalizeProfileLabel("   ") == null
            val validId = NativeCore.nativeIsValidProfileId("daily_profile-01")
            val rejectsUnsafeId = !NativeCore.nativeIsValidProfileId("../profile")
            val safeText = NativeCore.nativeIsSafeText("hello Android", 32)
            val rejectsControlText = !NativeCore.nativeIsSafeText("hello\nworld", 32)
            val checks = listOf(version == 2, capabilities == 15, sum == 42, normalizedLabel == "Daily Profile", rejectsEmptyLabel, validId, rejectsUnsafeId, safeText, rejectsControlText)
            val passed = checks.count { it }
            val status = if (passed == checks.size) "SEMUA TES BERHASIL" else "ADA TES GAGAL"
            "$status\nTes: $passed/${checks.size}\nAPI: v$version\nCapabilities: 0x${capabilities.toString(16)}\n20 + 22 = $sum\nLabel: $normalizedLabel\nProfile ID valid: $validId\nInput aman: $safeText"
        } catch (error: UnsatisfiedLinkError) {
            "GAGAL MEMUAT LIBRARY\n${error.localizedMessage ?: "JNI library tidak ditemukan."}"
        } catch (error: Exception) {
            "PENGUJIAN GAGAL\n${error.localizedMessage ?: error.javaClass.simpleName}"
        }
    }
}
