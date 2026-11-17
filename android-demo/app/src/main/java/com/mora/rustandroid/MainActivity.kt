package com.mora.rustandroid

import android.app.Activity
import android.content.Context
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.TrafficStats
import android.os.Process
import android.os.Build
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
    private val networkFile by lazy { java.io.File(filesDir, "network_snapshots.json") }
    private val securityAuditFile by lazy { java.io.File(filesDir, "security_audit.json") }

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
        column.addView(label("Profile · Configuration · Runtime Manager · Event Logger · Network Monitor · Security Auditor · Unified API v9"))

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

        section("Network Monitor")
        column.addView(label("Statistik byte untuk UID aplikasi ini saja; bukan trafik aplikasi lain."))
        column.addView(button("Ambil Snapshot Jaringan") { captureNetworkSnapshot() })
        column.addView(button("Lihat Riwayat Jaringan") { showNetworkHistory() })
        column.addView(button("Hapus Riwayat Jaringan") { clearNetworkHistory() })

        section("Security & Permission Auditor")
        column.addView(label("Audit metadata dan permission aplikasi ini saja; tidak memindai aplikasi lain atau mengubah permission."))
        column.addView(button("Jalankan Security Audit") { runSecurityAudit() })
        column.addView(button("Lihat Audit Terakhir") { showSecurityAudit() })

        section("Unified Core API")
        column.addView(label("Satu dispatcher JSON untuk semua modul Rust; API lama tetap kompatibel."))
        column.addView(button("Lihat Core API Descriptor") { showCoreApiDescriptor() })
        column.addView(button("Tes Unified Core API") { runUnifiedCoreApiDemo() })

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

    private fun captureNetworkSnapshot() {
        try {
            val connectivity = getSystemService(Context.CONNECTIVITY_SERVICE) as ConnectivityManager
            val network = connectivity.activeNetwork
            val capabilities = network?.let { connectivity.getNetworkCapabilities(it) }
            val transport = when {
                capabilities == null -> "none"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) -> "wifi"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) -> "cellular"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET) -> "ethernet"
                capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN) -> "vpn"
                else -> "other"
            }
            val rx = TrafficStats.getUidRxBytes(Process.myUid())
            val tx = TrafficStats.getUidTxBytes(Process.myUid())
            val snapshot = JSONObject().apply {
                put("uid", Process.myUid())
                put("rx_bytes", if (rx >= 0) rx else JSONObject.NULL)
                put("tx_bytes", if (tx >= 0) tx else JSONObject.NULL)
                put("transport", transport)
                put("connected", capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_INTERNET) == true)
                put("validated", capabilities?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) == true)
                put("metered", connectivity.isActiveNetworkMetered)
            }
            val response = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                readFile(networkFile, ""), "record", snapshot.toString(), System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                output.text = "Network monitor gagal: " + response.optString("error")
                return
            }
            val store = response.getJSONObject("monitor")
            networkFile.writeText(store.toString(), Charsets.UTF_8)
            val samples = store.getJSONArray("snapshots")
            val latest = samples.getJSONObject(samples.length() - 1)
            recordEvent("system", "network_snapshot", "Transport: $transport")
            output.text = "Snapshot jaringan tersimpan.\nUID aplikasi: " + Process.myUid() +
                "\nTransport: " + transport +
                "\nTerhubung: " + latest.optBoolean("connected") +
                "\nTervalidasi: " + latest.optBoolean("validated") +
                "\nMetered: " + latest.optBoolean("metered") +
                "\nRX total: " + latest.opt("rx_bytes") +
                "\nTX total: " + latest.opt("tx_bytes") +
                "\nRX sejak snapshot sebelumnya: " + latest.opt("rx_delta_bytes") +
                "\nTX sejak snapshot sebelumnya: " + latest.opt("tx_delta_bytes")
        } catch (e: Exception) {
            output.text = "Network monitor error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun showNetworkHistory() {
        try {
            val response = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                readFile(networkFile, ""), "summary", "", System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                output.text = "Riwayat jaringan gagal: " + response.optString("error")
                return
            }
            val store = response.getJSONObject("monitor")
            val samples = store.getJSONArray("snapshots")
            val summary = response.getJSONObject("summary")
            val lines = (0 until samples.length()).map { index ->
                val item = samples.getJSONObject(index)
                "#" + item.optLong("id") + " " + item.optString("transport") +
                    " · RX +" + item.opt("rx_delta_bytes") +
                    " · TX +" + item.opt("tx_delta_bytes") +
                    if (item.optBoolean("counter_reset")) " · counter reset" else ""
            }
            output.text = "Snapshot: " + summary.optInt("samples") +
                "\nTotal RX terukur: " + summary.optLong("total_rx_delta_bytes") + " bytes" +
                "\nTotal TX terukur: " + summary.optLong("total_tx_delta_bytes") + " bytes\n\n" +
                lines.reversed().joinToString("\n").ifEmpty { "Belum ada snapshot." }
        } catch (e: Exception) {
            output.text = "Riwayat jaringan error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun clearNetworkHistory() {
        try {
            val response = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                readFile(networkFile, ""), "clear", "", System.currentTimeMillis()
            ))
            if (!response.optBoolean("ok")) {
                output.text = "Hapus riwayat gagal: " + response.optString("error")
                return
            }
            networkFile.writeText(response.getJSONObject("monitor").toString(), Charsets.UTF_8)
            recordEvent("system", "network_history_cleared", "Riwayat jaringan dibersihkan")
            output.text = "Riwayat jaringan berhasil dibersihkan."
        } catch (e: Exception) {
            output.text = "Hapus riwayat error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runSecurityAudit() {
        try {
            @Suppress("DEPRECATION")
            val packageInfo = packageManager.getPackageInfo(packageName, PackageManager.GET_PERMISSIONS)
            val permissions = org.json.JSONArray()
            (packageInfo.requestedPermissions ?: emptyArray()).forEach { permission ->
                permissions.put(JSONObject().apply {
                    put("name", permission)
                    put("granted", checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED)
                })
            }
            val appInfo = applicationInfo
            val input = JSONObject().apply {
                put("package_name", packageName)
                put("target_sdk", appInfo.targetSdkVersion)
                put("debuggable", appInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE != 0)
                put("allow_backup", appInfo.flags and 0x00008000 != 0)
                put("uses_cleartext_traffic", appInfo.flags and 0x08000000 != 0)
                put("security_patch", Build.VERSION.SECURITY_PATCH ?: "")
                put("permissions", permissions)
            }
            val response = JSONObject(NativeCore.nativeSecurityAudit(input.toString(), System.currentTimeMillis()))
            if (!response.optBoolean("ok")) {
                output.text = "Security audit gagal: " + response.optString("error")
                return
            }
            val report = response.getJSONObject("report")
            securityAuditFile.writeText(report.toString(2), Charsets.UTF_8)
            val findings = report.getJSONArray("findings")
            val lines = (0 until findings.length()).map { index ->
                val item = findings.getJSONObject(index)
                "[" + item.optString("severity").uppercase() + "] " + item.optString("title") +
                    "\n" + item.optString("detail") +
                    "\nSaran: " + item.optString("recommendation")
            }
            output.text = "SECURITY AUDIT SELESAI" +
                "\nRisk: " + report.optString("risk_level").uppercase() +
                " (" + report.optInt("risk_score") + "/100)" +
                "\nPackage: " + report.optString("package_name") +
                "\nTarget SDK: " + appInfo.targetSdkVersion +
                "\nSecurity patch: " + Build.VERSION.SECURITY_PATCH +
                "\nPermission diminta: " + report.optInt("permission_count") +
                "\nDangerous granted: " + report.optInt("granted_dangerous_count") +
                "\nDangerous denied: " + report.optInt("denied_dangerous_count") +
                "\n\n" + lines.joinToString("\n\n").ifEmpty { "Tidak ada temuan." }
            recordEvent("security", "audit_completed", "Risk: " + report.optString("risk_level"))
        } catch (e: Exception) {
            output.text = "Security audit error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun showSecurityAudit() {
        if (!securityAuditFile.exists()) {
            output.text = "Belum ada hasil security audit."
            return
        }
        try {
            output.text = JSONObject(securityAuditFile.readText(Charsets.UTF_8)).toString(2)
        } catch (e: Exception) {
            output.text = "Gagal membaca audit: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun showCoreApiDescriptor() {
        try {
            output.text = JSONObject(NativeCore.nativeCoreDescriptor()).toString(2)
        } catch (e: Exception) {
            output.text = "Core API descriptor error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runUnifiedCoreApiDemo() {
        try {
            val request = JSONObject().apply {
                put("schema_version", 1)
                put("request_id", "android-demo")
                put("module", "profile_store")
                put("operation", "create")
                put("timestamp_ms", System.currentTimeMillis())
                put("state_json", """{"profiles":[]}""")
                put("params", JSONObject().put("id", "unified_demo").put("label", "Unified API"))
            }
            val response = JSONObject(NativeCore.nativeCoreDispatch(request.toString()))
            if (!response.optBoolean("ok")) {
                output.text = "Unified API gagal: " + response.optString("error")
                return
            }
            val created = response.getJSONObject("data").getJSONObject("store")
                .getJSONArray("profiles").getJSONObject(0)
            output.text = "UNIFIED CORE API BERHASIL" +
                "\nAPI: " + response.optInt("api_version") +
                "\nRequest ID: " + response.optString("request_id") +
                "\nModule: " + response.optString("module") +
                "\nProfile: " + created.optString("label")
        } catch (e: Exception) {
            output.text = "Unified API error: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }

    private fun runTests() {
        try {
            val checks = mutableListOf(
                NativeCore.nativeCoreApiVersion() == 9,
                NativeCore.nativeCoreCapabilities() == 2047,
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
            val emptyNetwork = JSONObject(NativeCore.nativeNetworkMonitorOperation("", "list", "", 0))
            checks.add(emptyNetwork.optBoolean("ok") &&
                emptyNetwork.getJSONObject("monitor").getJSONArray("snapshots").length() == 0)
            val networkSample = """{"uid":10123,"rx_bytes":1000,"tx_bytes":2000,"transport":"wifi","connected":true,"validated":true,"metered":false}"""
            val firstNetwork = JSONObject(NativeCore.nativeNetworkMonitorOperation("", "record", networkSample, 10))
            checks.add(firstNetwork.optBoolean("ok") &&
                firstNetwork.getJSONObject("monitor").getJSONArray("snapshots").length() == 1)
            val nextNetwork = """{"uid":10123,"rx_bytes":1300,"tx_bytes":2600,"transport":"wifi","connected":true,"validated":true,"metered":false}"""
            val secondNetwork = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                firstNetwork.getJSONObject("monitor").toString(), "record", nextNetwork, 20
            ))
            val latestNetwork = secondNetwork.getJSONObject("monitor").getJSONArray("snapshots").getJSONObject(1)
            checks.add(secondNetwork.optBoolean("ok") && latestNetwork.optLong("rx_delta_bytes") == 300L &&
                latestNetwork.optLong("tx_delta_bytes") == 600L)
            val invalidNetwork = """{"uid":10123,"rx_bytes":1,"tx_bytes":1,"transport":"bluetooth","connected":true,"validated":true,"metered":false}"""
            checks.add(!JSONObject(NativeCore.nativeNetworkMonitorOperation("", "record", invalidNetwork, 1)).optBoolean("ok"))
            val resetNetwork = """{"uid":10123,"rx_bytes":10,"tx_bytes":20,"transport":"wifi","connected":true,"validated":true,"metered":false}"""
            val resetResult = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                firstNetwork.getJSONObject("monitor").toString(), "record", resetNetwork, 30
            ))
            checks.add(resetResult.getJSONObject("monitor").getJSONArray("snapshots")
                .getJSONObject(1).optBoolean("counter_reset"))
            val clearedNetwork = JSONObject(NativeCore.nativeNetworkMonitorOperation(
                secondNetwork.getJSONObject("monitor").toString(), "clear", "", 40
            ))
            checks.add(clearedNetwork.optBoolean("ok") &&
                clearedNetwork.getJSONObject("monitor").getJSONArray("snapshots").length() == 0)
            val cleanAudit = JSONObject(NativeCore.nativeSecurityAudit(
                """{"package_name":"com.example.app","target_sdk":35,"debuggable":false,"allow_backup":false,"uses_cleartext_traffic":false,"security_patch":"2026-10-01","permissions":[]}""",
                10
            ))
            checks.add(cleanAudit.optBoolean("ok") &&
                cleanAudit.getJSONObject("report").optString("risk_level") == "good")
            val riskyAudit = JSONObject(NativeCore.nativeSecurityAudit(
                """{"package_name":"com.example.app","target_sdk":28,"debuggable":true,"allow_backup":false,"uses_cleartext_traffic":true,"security_patch":"","permissions":[]}""",
                20
            ))
            checks.add(riskyAudit.optBoolean("ok") &&
                riskyAudit.getJSONObject("report").optInt("risk_score") == 80)
            val descriptor = JSONObject(NativeCore.nativeCoreDescriptor())
            checks.add(descriptor.optInt("api_version") == 9 &&
                descriptor.getJSONObject("capabilities").optInt("mask") == 2047 &&
                descriptor.getJSONArray("modules").length() == 6)
            val unifiedRequest = JSONObject().apply {
                put("schema_version", 1)
                put("request_id", "smoke-test")
                put("module", "profile_store")
                put("operation", "create")
                put("timestamp_ms", 10)
                put("state_json", """{"profiles":[]}""")
                put("params", JSONObject().put("id", "api_test").put("label", "API Test"))
            }
            val unifiedResponse = JSONObject(NativeCore.nativeCoreDispatch(unifiedRequest.toString()))
            checks.add(unifiedResponse.optBoolean("ok") &&
                unifiedResponse.optInt("api_version") == 9 &&
                unifiedResponse.optString("request_id") == "smoke-test" &&
                unifiedResponse.getJSONObject("data").getJSONObject("store")
                    .getJSONArray("profiles").length() == 1)
            val badUnifiedRequest = JSONObject(unifiedRequest.toString()).put("module", "unknown_module")
            checks.add(!JSONObject(NativeCore.nativeCoreDispatch(badUnifiedRequest.toString())).optBoolean("ok"))
            val passed = checks.count { it }
            output.text = (if (passed == checks.size) "SEMUA TES BERHASIL" else "ADA TES GAGAL") +
                "\nTes: $passed/${checks.size}\nAPI v9 · Capability 0x7ff"
        } catch (e: Exception) {
            output.text = "Tes gagal: " + (e.localizedMessage ?: e.javaClass.simpleName)
        }
    }
}
