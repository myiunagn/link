import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.PrintWriter
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement

fun main() {
    val reader = BufferedReader(InputStreamReader(System.`in`))
    val input = reader.readLine() ?: "{}"
    
    try {
        val json = Json.parseToJsonElement(input).jsonObject
        val function = json["function"]?.jsonPrimitive?.content ?: ""
        val args = json["args"]?.jsonArray ?: JsonArray(emptyList())
        
        val result = when (function) {
            "add" -> {
                if (args.size < 2) {
                    mapOf("error" to "add requires 2 arguments")
                } else {
                    val a = toInt(args[0])
                    val b = toInt(args[1])
                    mapOf("result" to (a + b))
                }
            }
            "subtract" -> {
                if (args.size < 2) {
                    mapOf("error" to "subtract requires 2 arguments")
                } else {
                    val a = toInt(args[0])
                    val b = toInt(args[1])
                    mapOf("result" to (a - b))
                }
            }
            "multiply" -> {
                if (args.size < 2) {
                    mapOf("error" to "multiply requires 2 arguments")
                } else {
                    val a = toInt(args[0])
                    val b = toInt(args[1])
                    mapOf("result" to (a * b))
                }
            }
            "greet" -> {
                if (args.isEmpty()) {
                    mapOf("error" to "greet requires 1 argument")
                } else {
                    val name = args[0].jsonPrimitive.contentOrNull ?: args[0].toString()
                    mapOf("result" to "Hello from Kotlin, $name!")
                }
            }
            else -> mapOf("error" to "Unknown function: $function")
        }
        
        val out = PrintWriter(System.out, true)
        out.println(Json.encodeToString(JsonElement.serializer(), JsonObject(result.map { 
            it.key to JsonPrimitive(it.value.toString()) 
        })))
    } catch (e: Exception) {
        val out = PrintWriter(System.out, true)
        out.println("""{"error": "${e.message}"}""")
    }
}

fun toInt(json: JsonElement): Long {
    return when (json) {
        is JsonPrimitive -> json.content.toLongOrNull() ?: json.content.toDoubleOrNull()?.toLong() ?: 0L
        else -> 0L
    }
}