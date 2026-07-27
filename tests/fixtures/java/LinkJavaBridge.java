/// LinkJavaBridge - Java FFI 桥接器
/// 
/// 用法:
///   java LinkJavaBridge call <ClassName> <MethodName> <jsonRequest>
///   java LinkJavaBridge <ClassName>   // 持续运行模式
///
/// 输入 JSON 格式:
///   { "class": "...", "method": "...", "args": [v1, v2, ...] }
/// 输出 JSON 格式:
///   { "result": <value> } 或 { "error": "msg" }
public class LinkJavaBridge {
    public static void main(String[] args) throws Exception {
        if (args.length >= 1 && "call".equals(args[0])) {
            if (args.length < 4) {
                System.out.println("{\"error\":\"Usage: java LinkJavaBridge call <Class> <Method> <jsonRequest>\"}");
                System.exit(1);
            }
            String className = args[1];
            String methodName = args[2];
            String jsonRequest = args[3];
            try {
                Object result = invokeStatic(className, methodName, parseArgs(jsonRequest));
                System.out.println(formatResult(result));
            } catch (Exception e) {
                System.out.println("{\"error\":\"" + escapeJson(e.getMessage()) + "\"}");
            }
        } else {
            System.out.println("{\"error\":\"Use 'call' subcommand\"}");
            System.exit(1);
        }
    }

    public static Object invokeStatic(String className, String methodName, Object[] args) throws Exception {
        Class<?> clazz = Class.forName(className);
        Method[] methods = clazz.getMethods();
        for (Method m : methods) {
            if (!m.getName().equals(methodName)) continue;
            if (m.getParameterCount() != args.length) continue;
            if (!java.lang.reflect.Modifier.isStatic(m.getModifiers())) continue;
            try {
                return m.invoke(null, args);
            } catch (java.lang.IllegalArgumentException ignored) {
            }
        }
        throw new RuntimeException("No matching static method " + methodName + " in " + className);
    }

    public static Object[] parseArgs(String json) {
        int argsStart = json.indexOf("\"args\"");
        if (argsStart < 0) return new Object[0];
        int lb = json.indexOf('[', argsStart);
        int rb = json.lastIndexOf(']');
        if (lb < 0 || rb < 0) return new Object[0];
        String arr = json.substring(lb + 1, rb).trim();
        if (arr.isEmpty()) return new Object[0];

        java.util.List<Object> out = new java.util.ArrayList<>();
        int i = 0;
        while (i < arr.length()) {
            while (i < arr.length() && (Character.isWhitespace(arr.charAt(i)) || arr.charAt(i) == ',')) i++;
            if (i >= arr.length()) break;
            char c = arr.charAt(i);
            if (c == '"') {
                int end = i + 1;
                StringBuilder sb = new StringBuilder();
                while (end < arr.length() && arr.charAt(end) != '"') {
                    if (arr.charAt(end) == '\\' && end + 1 < arr.length()) end++;
                    sb.append(arr.charAt(end));
                    end++;
                }
                out.add(sb.toString());
                i = end + 1;
            } else if (c == 't' || c == 'f') {
                if (arr.startsWith("true", i)) { out.add(Boolean.TRUE); i += 4; }
                else if (arr.startsWith("false", i)) { out.add(Boolean.FALSE); i += 5; }
                else throw new RuntimeException("Invalid token at " + i);
            } else if (c == 'n') {
                out.add(null);
                i += 4;
            } else if (c == '-' || Character.isDigit(c)) {
                int end = i;
                while (end < arr.length() && (Character.isDigit(arr.charAt(end)) || arr.charAt(end) == '.' || arr.charAt(end) == '-')) end++;
                String num = arr.substring(i, end);
                if (num.contains(".")) out.add(Double.parseDouble(num));
                else out.add(Long.parseLong(num));
                i = end;
            } else {
                throw new RuntimeException("Unexpected char '" + c + "' at " + i);
            }
        }
        return out.toArray();
    }

    public static String formatResult(Object o) {
        if (o == null) return "{\"result\":null}";
        if (o instanceof Boolean) return "{\"result\":" + o + "}";
        if (o instanceof Number) return "{\"result\":" + o + "}";
        if (o instanceof String) return "{\"result\":\"" + escapeJson((String) o) + "\"}";
        return "{\"result\":\"" + escapeJson(o.toString()) + "\"}";
    }

    public static String escapeJson(String s) {
        if (s == null) return "";
        return s.replace("\\", "\\\\").replace("\"", "\\\"")
                .replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t");
    }
}
