<?php
$input = file_get_contents('php://stdin');
$data = json_decode($input, true);

if (!$data || !isset($data['function'])) {
    echo json_encode(['error' => 'Invalid request']);
    exit(1);
}

$function = $data['function'];
$args = isset($data['args']) ? $data['args'] : [];

$result = null;
switch ($function) {
    case 'add':
        if (count($args) < 2) {
            echo json_encode(['error' => 'add requires 2 arguments']);
            exit(1);
        }
        $result = $args[0] + $args[1];
        break;
    case 'subtract':
        if (count($args) < 2) {
            echo json_encode(['error' => 'subtract requires 2 arguments']);
            exit(1);
        }
        $result = $args[0] - $args[1];
        break;
    case 'multiply':
        if (count($args) < 2) {
            echo json_encode(['error' => 'multiply requires 2 arguments']);
            exit(1);
        }
        $result = $args[0] * $args[1];
        break;
    case 'greet':
        if (count($args) < 1) {
            echo json_encode(['error' => 'greet requires 1 argument']);
            exit(1);
        }
        $name = is_string($args[0]) ? $args[0] : strval($args[0]);
        $result = "Hello from PHP, $name!";
        break;
    default:
        echo json_encode(['error' => "Unknown function: $function"]);
        exit(1);
}

echo json_encode(['result' => $result]);