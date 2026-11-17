package com.mora.rustandroid

import android.app.Activity
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import org.json.JSONObject

class MainActivity : Activity() {
    private lateinit var output: TextView
    private lateinit var idInput: EditText
    private lateinit var labelInput: EditText
    private lateinit var configKeyInput: EditText
    private lateinit var configValueInput: EditText
    private lateinit var serviceInput: EditText
    private val runtimeFile by lazy { java.io.File(filesDir, "runtime_services.json") }
    private val profileFile by lazy { java.io.File(filesDir, "profiles.json") }
    private val configFile by lazy { java.io.File(filesDir, "system_config.json") }
    private val eventFile by lazy { java.io.File(filesDir, "activity_events.json") }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val column = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(24, 28, 24, 28)
        }
        fun label(text: String, size: Float = 15f) = TextView(this).apply {
            this.text = text
            textSize = size
            gravity = Gravity.CENTER
        }
        fun section(text: String) {
            column.addView(label(text, 19f).apply {
                typeface = Typeface.DEFAULT_BOLD
                setPadding(0, 24, 0, 8)
            })
        }

        column.addView(label("Mora Rust Android Core", 24f).apply {
            typeface = Typeface.DEFAULT_BOLD
        })
        column.addView(label("Profile · Configuration · Runtime Manager · Event Logger · JNI API v6"))

        section("Profile Manager")
        idInput = EditText(this).apply { hint = "Profile ID, contoh: daily_01" }
        labelInput = EditText(this).apply { hint = "Nama profil" }
        column.addView(idInput)
        column.addView(labelInput)
        column.addView(button("Buat Profil") { runProfile("create") })
        column.addView(button("Ubah Profil") { runProfile("update") })
        column.addView(button("Hapus Profil") { runProfile("delete") })
        column.addView(button("Muat / Lihat Profil") { runProfile("list") })

        section("System Configuration")
        column.addView(label("Key: log_level, network_mode, background_sync, cache_limit_mb, telemetry_enabled"))
        configKeyInput = EditText(this).apply { hint = "Config key, contoh: log_level" }
        configValueInput = EditText(this).apply { hint = "Value, contoh: debug" }
        column.addView(configKeyInput)
        column.addView(configValueInput)
        column.addView(button("Simpan Pengaturan") { runConfig("set") })
        column.addView(button("Lihat Pengaturan") { runConfig("get") })
        column.addView(button("Reset Pengaturan") { runConfig("reset") })

        section("Runtime & Service Manager")
        column.addView(label("Service ID: profile_store · config_store · diagnostics"))
        serviceInput = EditText(this).apply {
            hint = "Service ID: profile_store / config_store / diagnostics"
            setText("diagnostics")
        }
        column.addView(serviceInput)
        column.addView(button("Lihat Status Semua Service") { runRuntime("list") })
        column.addView(button("Start Service") { runRuntime("start") })
        column.addView(button("Stop Service") { runRuntime("stop") })
        column.addView(button("Enable Service") { runRuntime("enable") })
        column.addView(button("Disable Service") { runRuntime("disable") })

        section("Event & Activity Logger")
        column.addView(button("Lihat Activity Log") { showEventLog() })
        column.addView(button("Hapus Semua Log") { clearEventLog() })

        section("Diagnostics")
        column.addView(Button(this).apply {
            text = "Jalankan Semua Rust Tests"
            setOnClickListener { runTests() }
        })
        output = label("Core siap.")
        output.gravity = Gravity.START
        output.setPadding(0, 20, 0, 0)
        column.addView(output)
        setContentView(ScrollView(this).apply { addView(column) })
        runProfile("list")
        runConfig("get")
        runRuntime("list")
        recordEvent("system", "app_open", "Mora Rust Android Core dibuka")
    }

    private fun button(title: String, action: () -> Unit) =
        Button(this).apply {
            text = title
            setOnClickListener { action() }
        }

    private fun readFile(file: java.io.File, fallback: String): String =
        if (file.exists()) file.readText(Charsets.UTF_8) else fallback

    
    private fun recordEvent(category: String, action: String, detail: String) {
        try {
            val response = JSONObject(NativeCore.nativeEventLogOperation(
                readFile(eventFile, ""), "record", category, action,
                detail.take(240).ifBlank { "No detail" }, System.currentTimeMillis()
            ))
            if (response.optBoolean("ok")) {
                eventFile.writeText(response.getJSONObject("log").toString(), Charsets.UTF_8)
            }
        } catch (_: Exception) {
            // Logging must never interrupt the primary operation.
        }
    }

    private fun showEventLog() {
        try {
            val response = JSONObject(NativeCore.nativeEventLogOperation(
                readFile(eventFile, ""), "list", "", "", "", System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                output.text = "Event log gagal: " + response.optString("error")
                return
            }
            val events = response.getJSONObject("log").getJSONArray("events")
            output.text = (0 until events.length()).map { i ->
                val event = events.getJSONObject(i)
                "#" + event.optLong("id") + " " + event.optString("category") + "/" +
                    event.optString("action") + ": " + event.optString("detail")
            }.reversed().joinToString("\n")
            if (events.length() == 0) output.text = "Belum ada event."
        } catch (e: Exception) {
            output.text = "Event log error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun clearEventLog() {
        try {
            val response = JSONObject(NativeCore.nativeEventLogOperation(
                readFile(eventFile, ""), "clear", "", "", "", System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                output.text = "Hapus log gagal: " + response.optString("error")
                return
            }
            eventFile.writeText(response.getJSONObject("log").toString(), Charsets.UTF_8)
            recordEvent("system", "event_log_cleared", "Activity log dibersihkan")
            output.text = "Activity log berhasil dibersihkan."
        } catch (e: Exception) {
            output.text = "Hapus log error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runProfile(operation: String) {
        try {
            val response = JSONObject(
                NativeCore.nativeProfileOperation(
                    readFile(profileFile, """{"profiles":[]}"""),
                    operation,
                    idInput.text.toString().trim(),
                    labelInput.text.toString(),
                    System.currentTimeMillis()
                )
            )
            if (!response.optBoolean("ok")) {
                recordEvent("profile", operation + "_failed", response.optString("error"))
                output.text = "Profile gagal: " + response.optString("error")
                return
            }
            val json = response.getJSONObject("store").toString(2)
            profileFile.writeText(json, Charsets.UTF_8)
            recordEvent("profile", operation, "Operasi profile berhasil")
            output.text = "Profile berhasil: $operation\n$json"
        } catch (e: Exception) {
            recordEvent("profile", "exception", e.javaClass.simpleName)
            output.text = "Profile error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runConfig(operation: String) {
        try {
            val response = JSONObject(
                NativeCore.nativeSystemConfigOperation(
                    readFile(configFile, ""),
                    operation,
                    configKeyInput.text.toString().trim(),
                    configValueInput.text.toString().trim(),
                    System.currentTimeMillis()
                )
            )
            if (!response.optBoolean("ok")) {
                recordEvent("config", operation + "_failed", response.optString("error"))
                output.text = "Config gagal: " + response.optString("error")
                return
            }
            val json = response.getJSONObject("config").toString(2)
            configFile.writeText(json, Charsets.UTF_8)
            recordEvent("config", operation, "Operasi configuration berhasil")
            output.text = "System config berhasil: $operation\n$json"
        } catch (e: Exception) {
            recordEvent("config", "exception", e.javaClass.simpleName)
            output.text = "Config error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }


    private fun runRuntime(operation: String) {
        try {
            val response = JSONObject(NativeCore.nativeRuntimeOperation(
                readFile(runtimeFile, ""), operation,
                serviceInput.text.toString().trim().ifEmpty { "diagnostics" },
                System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                recordEvent("runtime", operation + "_failed", response.optString("error"))
                output.text = "Runtime gagal: " + response.optString("error")
                return
            }
            val json = response.getJSONObject("runtime").toString(2)
            runtimeFile.writeText(json, Charsets.UTF_8)
            recordEvent("runtime", operation, "Service: " + serviceInput.text.toString().trim())
            output.text = "Runtime berhasil: $operation\\n$json\\nStatus logis service internal aplikasi."
        } catch (e: Exception) {
            recordEvent("runtime", "exception", e.javaClass.simpleName)
            output.text = "Runtime error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runTests() {
        try {
            val checks = mutableListOf(
                NativeCore.nativeCoreApiVersion() == 6,
                NativeCore.nativeCoreCapabilities() == 255,
                NativeCore.nativeAdd(20, 22) == 42,
                NativeCore.nativeNormalizeProfileLabel(" Daily ") == "Daily",
                NativeCore.nativeNormalizeProfileLabel("  ") == null,
                NativeCore.nativeIsValidProfileId("daily_01"),
                !NativeCore.nativeIsValidProfileId("../bad"),
                NativeCore.nativeIsSafeText("hello", 16),
                !NativeCore.nativeIsSafeText("bad\ntext", 16)
            )
            val created = JSONObject(
                NativeCore.nativeProfileOperation(
                    """{"profiles":[]}""", "create", "test_01", "Test Profile", 10
                )
            )
            checks.add(
                created.optBoolean("ok") &&
                    created.getJSONObject("store").getJSONArray("profiles").length() == 1
            )
            val updated = JSONObject(
                NativeCore.nativeProfileOperation(
                    created.getJSONObject("store").toString(), "update", "test_01", "Updated", 20
                )
            )
            checks.add(
                updated.optBoolean("ok") &&
                    updated.getJSONObject("store").getJSONArray("profiles")
                        .getJSONObject(0).getString("label") == "Updated"
            )
            val deleted = JSONObject(
                NativeCore.nativeProfileOperation(
                    updated.getJSONObject("store").toString(), "delete", "test_01", "", 30
                )
            )
            checks.add(
                deleted.optBoolean("ok") &&
                    deleted.getJSONObject("store").getJSONArray("profiles").length() == 0
            )
            val defaultConfig = JSONObject(
                NativeCore.nativeSystemConfigOperation("", "get", "", "", 0)
            )
            checks.add(
                defaultConfig.optBoolean("ok") &&
                    defaultConfig.getJSONObject("config").optString("log_level") == "info"
            )
            val changedConfig = JSONObject(
                NativeCore.nativeSystemConfigOperation("", "set", "log_level", "debug", 10)
            )
            checks.add(
                changedConfig.optBoolean("ok") &&
                    changedConfig.getJSONObject("config").optString("log_level") == "debug"
            )
            val invalidConfig = JSONObject(
                NativeCore.nativeSystemConfigOperation("", "set", "unknown_key", "x", 10)
            )
            checks.add(!invalidConfig.optBoolean("ok"))
            val resetConfig = JSONObject(
                NativeCore.nativeSystemConfigOperation("", "reset", "", "", 10)
            )
            checks.add(
                resetConfig.optBoolean("ok") &&
                    resetConfig.getJSONObject("config").optString("log_level") == "info"
            )

            val defaultRuntime = JSONObject(NativeCore.nativeRuntimeOperation("", "list", "", 0))
            checks.add(defaultRuntime.optBoolean("ok") &&
                defaultRuntime.getJSONObject("runtime").getJSONArray("services").length() == 3)
            val started = JSONObject(NativeCore.nativeRuntimeOperation("", "start", "diagnostics", 10))
            checks.add(started.optBoolean("ok") &&
                started.getJSONObject("runtime").getJSONArray("services").getJSONObject(2).optBoolean("running"))
            val disabled = JSONObject(NativeCore.nativeRuntimeOperation("", "disable", "diagnostics", 20))
            checks.add(disabled.optBoolean("ok") &&
                !disabled.getJSONObject("runtime").getJSONArray("services").getJSONObject(2).optBoolean("running"))
            val rejected = JSONObject(NativeCore.nativeRuntimeOperation(
                disabled.getJSONObject("runtime").toString(), "start", "diagnostics", 30
            ))
            checks.add(!rejected.optBoolean("ok"))
            val emptyLog = JSONObject(NativeCore.nativeEventLogOperation("", "list", "", "", "", 0))
            checks.add(emptyLog.optBoolean("ok") && emptyLog.getJSONObject("log").getJSONArray("events").length() == 0)
            val recordedLog = JSONObject(NativeCore.nativeEventLogOperation("", "record", "system", "test_event", "logger smoke test", 10))
            checks.add(recordedLog.optBoolean("ok") && recordedLog.getJSONObject("log").getJSONArray("events").length() == 1)
            val clearedLog = JSONObject(NativeCore.nativeEventLogOperation(recordedLog.getJSONObject("log").toString(), "clear", "", "", "", 20))
            checks.add(clearedLog.optBoolean("ok") && clearedLog.getJSONObject("log").getJSONArray("events").length() == 0)
            val invalidEvent = JSONObject(NativeCore.nativeEventLogOperation("", "record", "other_app", "read", "not allowed", 10))
            checks.add(!invalidEvent.optBoolean("ok"))
            val passed = checks.count { it }
            output.text = (if (passed == checks.size) "SEMUA TES BERHASIL" else "ADA TES GAGAL") +
                "\nTes: $passed/${checks.size}\nAPI v6 · Capability 0xff"
        } catch (e: Exception) {
            output.text = "Tes gagal: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }
}
