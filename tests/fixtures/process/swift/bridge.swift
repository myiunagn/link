import Foundation

struct Request: Codable {
    let module: String
    let function: String
    let args: [AnyCodable]
}

struct AnyCodable: Codable {
    let value: Any
    
    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let intVal = try? container.decode(Int.self) {
            value = intVal
        } else if let doubleVal = try? container.decode(Double.self) {
            value = doubleVal
        } else if let boolVal = try? container.decode(Bool.self) {
            value = boolVal
        } else if let strVal = try? container.decode(String.self) {
            value = strVal
        } else if container.decodeNil() {
            value = NSNull()
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unsupported type")
        }
    }
    
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch value {
        case let intVal as Int:
            try container.encode(intVal)
        case let doubleVal as Double:
            try container.encode(doubleVal)
        case let boolVal as Bool:
            try container.encode(boolVal)
        case let strVal as String:
            try container.encode(strVal)
        case is NSNull:
            try container.encodeNil()
        default:
            try container.encode(String(describing: value))
        }
    }
}

func toInt(_ value: Any) -> Int {
    if let intVal = value as? Int { return intVal }
    if let doubleVal = value as? Double { return Int(doubleVal) }
    if let strVal = value as? String { return Int(strVal) ?? 0 }
    return 0
}

func toString(_ value: Any) -> String {
    if let strVal = value as? String { return strVal }
    return String(describing: value)
}

let inputData = FileHandle.standardInput.readDataToEndOfFile()
let jsonStr = String(data: inputData, encoding: .utf8) ?? "{}"
let jsonData = jsonStr.data(using: .utf8)!

do {
    let req = try JSONDecoder().decode(Request.self, from: jsonData)
    let function = req.function
    let args = req.args.map { $0.value }
    
    var result: Any? = nil
    
    switch function {
    case "add":
        guard args.count >= 2 else {
            try? JSONEncoder().encode(["error": "add requires 2 arguments"]).write(to: FileHandle.standardOutput)
            exit(1)
        }
        result = toInt(args[0]) + toInt(args[1])
        
    case "subtract":
        guard args.count >= 2 else {
            try? JSONEncoder().encode(["error": "subtract requires 2 arguments"]).write(to: FileHandle.standardOutput)
            exit(1)
        }
        result = toInt(args[0]) - toInt(args[1])
        
    case "multiply":
        guard args.count >= 2 else {
            try? JSONEncoder().encode(["error": "multiply requires 2 arguments"]).write(to: FileHandle.standardOutput)
            exit(1)
        }
        result = toInt(args[0]) * toInt(args[1])
        
    case "greet":
        guard !args.isEmpty else {
            try? JSONEncoder().encode(["error": "greet requires 1 argument"]).write(to: FileHandle.standardOutput)
            exit(1)
        }
        result = "Hello from Swift, \(toString(args[0]))!"
        
    default:
        let errData = try JSONSerialization.data(withJSONObject: ["error": "Unknown function: \(function)"])
        FileHandle.standardOutput.write(errData)
        exit(1)
    }
    
    if let result = result {
        let respData = try JSONSerialization.data(withJSONObject: ["result": result])
        FileHandle.standardOutput.write(respData)
    }
} catch {
    let errData = try? JSONSerialization.data(withJSONObject: ["error": error.localizedDescription])
    if let errData = errData {
        FileHandle.standardOutput.write(errData)
    }
    exit(1)
}