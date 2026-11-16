package com.mora.rustandroid

import org.junit.Assert.assertEquals
import org.junit.Test

class NativeCoreSmokeTest {
    @Test
    fun loadsRustLibraryAndCallsJniExports() {
        assertEquals(1, NativeCore.nativeCoreApiVersion())
        assertEquals(42, NativeCore.nativeAdd(20, 22))
    }
}
