package com.example.simplecounter

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.burakguner.myapp.shared.handleResponse
import com.burakguner.myapp.shared.processEvent
import com.burakguner.myapp.shared.view
import com.crux.example.simple_counter.DelayOperation
import com.crux.example.simple_counter.DelayOutput
import com.crux.example.simple_counter.Effect
import com.crux.example.simple_counter.Event
import com.crux.example.simple_counter.Request
import com.crux.example.simple_counter.Requests
import com.crux.example.simple_counter.ViewModel
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.random.Random

class Core : androidx.lifecycle.ViewModel() {
    var view: ViewModel? by mutableStateOf(null)
        private set

    fun update(event: Event) {
        val effects = processEvent(event.bincodeSerialize())

        val requests = Requests.bincodeDeserialize(effects)
        for (request in requests) {
            processEffect(request)
        }
    }


    private fun processEffect(request: Request) {
        when (request.effect) {
            is Effect.Render -> {
                this.view = ViewModel.bincodeDeserialize(view())
            }
            is Effect.Delay -> {
                val effect = request.effect as Effect.Delay
                when (effect.value) {
                    is DelayOperation.Random -> {
                        val operation = effect.value as DelayOperation.Random
                        val randomValue = Random.nextLong(operation.field0, operation.field1)
                        val response = DelayOutput.Random(randomValue)
                        respond(request, response.bincodeSerialize())
                    }
                    is DelayOperation.Delay -> {
                        val operation = effect.value as DelayOperation.Delay
                        GlobalScope.launch {
                            delay(operation.value)
                            val response = DelayOutput.TimeUp()
                            respond(request, response.bincodeSerialize())
                        }
                    }
                }
            }
        }
    }

    private fun respond(request: Request, response: ByteArray) {
        val effects = handleResponse(request.id!!.toUInt(), response)
        val requests = Requests.bincodeDeserialize(effects)

        for (request in requests) {
            processEffect(request)
        }
    }
}