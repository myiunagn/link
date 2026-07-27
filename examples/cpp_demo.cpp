// C++ 示例库:演示 Link 通过 C ABI 调用 C++ 代码
// 编译(MSVC): cl /LD cpp_demo.cpp /link /OUT:cpp_demo.dll
// 编译(g++/clang++): g++ -shared -fPIC -o libcpp_demo.so cpp_demo.cpp
//
// 关键点:C++ 函数必须用 extern "C" 导出,以避免名称修饰(name mangling),
// 这样 Link 才能通过 dlopen/LoadLibrary + 符号名直接找到它们。

#include <cstdint>
#include <cmath>
#include <cstring>
#include <string>
#include <vector>
#include <algorithm>

// ---- 简单算术 ----

extern "C" __declspec(dllexport) int32_t cpp_add(int32_t a, int32_t b) {
    return a + b;
}

extern "C" __declspec(dllexport) int32_t cpp_sub(int32_t a, int32_t b) {
    return a - b;
}

extern "C" __declspec(dllexport) int32_t cpp_mul(int32_t a, int32_t b) {
    return a * b;
}

// ---- 浮点运算(使用 C++ <cmath>) ----

extern "C" __declspec(dllexport) double cpp_sqrt(double x) {
    return std::sqrt(x);
}

extern "C" __declspec(dllexport) double cpp_pow(double base, double exp) {
    return std::pow(base, exp);
}

// ---- 递归 ----

extern "C" __declspec(dllexport) int32_t cpp_factorial(int32_t n) {
    if (n <= 1) return 1;
    return n * cpp_factorial(n - 1);
}

extern "C" __declspec(dllexport) int32_t cpp_fib(int32_t n) {
    if (n < 2) return n;
    return cpp_fib(n - 1) + cpp_fib(n - 2);
}

// ---- 字符串返回(使用 C++ std::string 内部管理内存,返回 C 字符串) ----
// 注意:返回的指针必须指向静态或堆内存,调用方不释放(本示例用静态缓冲区)

extern "C" __declspec(dllexport) const char* cpp_version() {
    return "Link-C++ Bridge v1.0 (built with C++17)";
}

extern "C" __declspec(dllexport) const char* cpp_greet(const char* name) {
    static thread_local std::string buffer;
    buffer = "Hello, ";
    buffer += name;
    buffer += "! from C++";
    return buffer.c_str();
}

// ---- 布尔逻辑 ----

extern "C" __declspec(dllexport) bool cpp_is_even(int32_t n) {
    return (n % 2) == 0;
}

// ---- 使用 C++ STL 容器(对外通过 C ABI 暴露简单类型) ----

extern "C" __declspec(dllexport) int32_t cpp_max3(int32_t a, int32_t b, int32_t c) {
    // 内部使用 STL,对外仍是 C ABI
    std::vector<int32_t> v = {a, b, c};
    return *std::max_element(v.begin(), v.end());
}

extern "C" __declspec(dllexport) double cpp_average(int32_t a, int32_t b, int32_t c) {
    std::vector<int32_t> v = {a, b, c};
    int32_t sum = 0;
    for (auto x : v) sum += x;
    return static_cast<double>(sum) / v.size();
}
