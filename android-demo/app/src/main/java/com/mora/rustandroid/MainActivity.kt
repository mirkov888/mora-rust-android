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
            layoutParams = ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.MATCH_PARENT
            )
        }

        val title = TextView(this).apply {
            text = "Mora Rust Android Core"
            textSize = 24f
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
        }
        val description = TextView(this).apply {
            text = "Demo aplikasi untuk menguji komunikasi Kotlin ↔ Rust melalui JNI."
            textSize = 15f
            gravity = Gravity.CENTER
            setPadding(0, 16, 0, 24)
        }
        result = TextView(this).apply {
            text = "Belum menjalankan pengujian."
            textSize = 16f
            gravity = Gravity.CENTER
            setPadding(0, 16, 0, 24)
        }
        val testButton = Button(this).apply {
            text = "Jalankan Rust JNI Test"
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
            val sum = NativeCore.nativeAdd(20, 22)
            if (version == 1 && sum == 42) {
                "BERHASIL\nRust API: v$version\n20 + 22 = $sum\nJNI library berhasil dimuat."
            } else {
                "HASIL TIDAK SESUAI\nAPI: v$version\nHasil penjumlahan: $sum"
            }
        } catch (error: UnsatisfiedLinkError) {
            "GAGAL MEMUAT LIBRARY\n${error.localizedMessage ?: "JNI library tidak ditemukan."}"
        } catch (error: Exception) {
            "PENGUJIAN GAGAL\n${error.localizedMessage ?: error.javaClass.simpleName}"
        }
    }
}
