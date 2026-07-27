/// Java FFI 示例类
/// 演示如何暴露静态方法给 Link 调用
public class MathUtils {
    public static long add(long a, long b) {
        return a + b;
    }

    public static long multiply(long a, long b) {
        return a * b;
    }

    public static double divide(double a, double b) {
        return a / b;
    }

    public static String greet(String name) {
        return "Hello from Java, " + name + "!";
    }

    public static boolean isEven(long n) {
        return n % 2 == 0;
    }

    public static long power(long base, long exp) {
        long result = 1;
        for (long i = 0; i < exp; i++) result *= base;
        return result;
    }
}
