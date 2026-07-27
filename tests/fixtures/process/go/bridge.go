package main

import (
	"encoding/json"
	"fmt"
	"os"
)

type Request struct {
	Module   string          `json:"module"`
	Function string          `json:"function"`
	Args     json.RawMessage `json:"args"`
}

type Response struct {
	Result interface{} `json:"result,omitempty"`
	Error  string      `json:"error,omitempty"`
}

func add(args []interface{}) interface{} {
	if len(args) < 2 {
		return map[string]interface{}{"error": "add requires 2 arguments"}
	}
	a := toFloat(args[0])
	b := toFloat(args[1])
	if a == float64(int64(a)) && b == float64(int64(b)) {
		return int64(a) + int64(b)
	}
	return a + b
}

func subtract(args []interface{}) interface{} {
	if len(args) < 2 {
		return map[string]interface{}{"error": "subtract requires 2 arguments"}
	}
	a := toFloat(args[0])
	b := toFloat(args[1])
	if a == float64(int64(a)) && b == float64(int64(b)) {
		return int64(a) - int64(b)
	}
	return a - b
}

func multiply(args []interface{}) interface{} {
	if len(args) < 2 {
		return map[string]interface{}{"error": "multiply requires 2 arguments"}
	}
	a := toFloat(args[0])
	b := toFloat(args[1])
	if a == float64(int64(a)) && b == float64(int64(b)) {
		return int64(a) * int64(b)
	}
	return a * b
}

func greet(args []interface{}) interface{} {
	if len(args) < 1 {
		return map[string]interface{}{"error": "greet requires 1 argument"}
	}
	name := fmt.Sprintf("%v", args[0])
	return fmt.Sprintf("Hello from Go, %s!", name)
}

func toFloat(v interface{}) float64 {
	switch val := v.(type) {
	case float64:
		return val
	case int64:
		return float64(val)
	case json.Number:
		f, _ := val.Float64()
		return f
	default:
		return 0
	}
}

func main() {
	var req Request
	decoder := json.NewDecoder(os.Stdin)
	if err := decoder.Decode(&req); err != nil {
		resp := Response{Error: fmt.Sprintf("Failed to parse request: %v", err)}
		json.NewEncoder(os.Stdout).Encode(resp)
		return
	}

	var args []interface{}
	if err := json.Unmarshal(req.Args, &args); err != nil {
		resp := Response{Error: fmt.Sprintf("Failed to parse args: %v", err)}
		json.NewEncoder(os.Stdout).Encode(resp)
		return
	}

	var result interface{}
	switch req.Function {
	case "add":
		result = add(args)
	case "subtract":
		result = subtract(args)
	case "multiply":
		result = multiply(args)
	case "greet":
		result = greet(args)
	default:
		resp := Response{Error: fmt.Sprintf("Unknown function: %s", req.Function)}
		json.NewEncoder(os.Stdout).Encode(resp)
		return
	}

	if errResult, ok := result.(map[string]interface{}); ok {
		if errMsg, ok := errResult["error"]; ok {
			resp := Response{Error: fmt.Sprintf("%v", errMsg)}
			json.NewEncoder(os.Stdout).Encode(resp)
			return
		}
	}

	resp := Response{Result: result}
	json.NewEncoder(os.Stdout).Encode(resp)
}