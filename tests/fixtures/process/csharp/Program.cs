using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;

class Program
{
    static int Main()
    {
        try
        {
            string input = Console.In.ReadToEnd();
            using JsonDocument doc = JsonDocument.Parse(input);
            JsonElement root = doc.RootElement;

            string function = root.GetProperty("function").GetString();
            JsonElement args = root.GetProperty("args");

            JsonElement result = function switch
            {
                "add" => HandleAdd(args),
                "subtract" => HandleSubtract(args),
                "multiply" => HandleMultiply(args),
                "greet" => HandleGreet(args),
                _ => JsonSerializer.SerializeToElement(new { error = $"Unknown function: {function}" })
            };

            using var stream = new MemoryStream();
            using var writer = new Utf8JsonWriter(stream);
            writer.WriteStartObject();
            writer.WritePropertyName("result");
            result.WriteTo(writer);
            writer.WriteEndObject();
            writer.Flush();
            Console.WriteLine(System.Text.Encoding.UTF8.GetString(stream.ToArray()));
            return 0;
        }
        catch (Exception ex)
        {
            Console.WriteLine(JsonSerializer.Serialize(new { error = ex.Message }));
            return 1;
        }
    }

    static JsonElement HandleAdd(JsonElement args)
    {
        if (args.GetArrayLength() < 2)
            return JsonSerializer.SerializeToElement(new { error = "add requires 2 arguments" });
        long a = args[0].GetInt64();
        long b = args[1].GetInt64();
        return JsonSerializer.SerializeToElement(a + b);
    }

    static JsonElement HandleSubtract(JsonElement args)
    {
        if (args.GetArrayLength() < 2)
            return JsonSerializer.SerializeToElement(new { error = "subtract requires 2 arguments" });
        long a = args[0].GetInt64();
        long b = args[1].GetInt64();
        return JsonSerializer.SerializeToElement(a - b);
    }

    static JsonElement HandleMultiply(JsonElement args)
    {
        if (args.GetArrayLength() < 2)
            return JsonSerializer.SerializeToElement(new { error = "multiply requires 2 arguments" });
        long a = args[0].GetInt64();
        long b = args[1].GetInt64();
        return JsonSerializer.SerializeToElement(a * b);
    }

    static JsonElement HandleGreet(JsonElement args)
    {
        if (args.GetArrayLength() < 1)
            return JsonSerializer.SerializeToElement(new { error = "greet requires 1 argument" });
        string name = args[0].GetString() ?? "unknown";
        return JsonSerializer.SerializeToElement($"Hello from C#, {name}!");
    }
}