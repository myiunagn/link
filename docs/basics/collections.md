# 列表与字符串

## 列表

### 创建列表

```link
let nums = [1, 2, 3, 4, 5];
let strings = ["a", "b", "c"];
let floats = [1.1, 2.2, 3.3];
let bools = [true, false, true];
let empty = [];

// 异构列表(可包含不同类型)
let mixed = [1, "two", 3.0, true, none];

// 嵌套列表
let matrix = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];
```

### 索引访问

从 0 开始,用 `[]` 访问:

```link
let nums = [10, 20, 30, 40, 50];

println(nums[0]);    // 10
println(nums[1]);    // 20
println(nums[4]);    // 50

// 越界会报错
// println(nums[10]);  // Error: Index 10 out of bounds for list of length 5
```

### 嵌套索引

```link
let matrix = [[1, 2], [3, 4], [5, 6]];

println(matrix[0]);       // [1, 2]
println(matrix[0][0]);    // 1
println(matrix[2][1]);    // 6
```

### 列表长度

用 `len()` 函数:

```link
let nums = [1, 2, 3, 4, 5];
let n = len(nums);
println(n);    // 5

let empty = [];
println(len(empty));   // 0
```

### 遍历列表

```link
let fruits = ["苹果", "香蕉", "橙子"];

for i in 0..len(fruits) {
    println("第", i + 1, "个:", fruits[i]);
}
```

输出:

```
第 1 个:苹果
第 2 个:香蕉
第 3 个:橙子
```

### 修改元素

通过索引赋值:

```link
let nums = [1, 2, 3];
nums[0] = 100;
nums[2] = 300;
println(nums);   // [100, 2, 300]
```

!!! note "v0.1 限制"
    v0.1 暂不支持 `push` / `pop` / `append` / `insert` 等修改列表大小的方法。
    后续版本会加入。当前列表大小是固定的。

## 字符串

### 创建字符串

```link
let greeting = "Hello, World!";
let empty = "";
let chinese = "你好,世界";
let escaped = "She said \"hi\"";
let path = "C:\\Users\\name";
```

### 字符串长度

```link
let s = "hello";
println(len(s));   // 5

let cn = "你好";
println(len(cn));   // 2 (按 UTF-8 字符计)
```

### 字符串索引

按字符位置(非字节)索引,返回单字符字符串:

```link
let s = "hello";
println(s[0]);    // h
println(s[1]);    // e
println(s[4]);    // o

let cn = "你好世界";
println(cn[0]);   // 你
println(cn[2]);   // 世
```

### 遍历字符串

```link
let s = "abc";
for i in 0..len(s) {
    println(s[i]);
}
// 输出:
// a
// b
// c
```

### 字符串拼接

用 `+` 拼接:

```link
let first = "Hello";
let second = "World";
let combined = first + ", " + second + "!";
println(combined);   // Hello, World!
```

### 字符串与内置函数

```link
let s = "Hello";
println(len(s));     // 5
println(s);          // Hello
print(s, " ");       // Hello (不换行)
print("!");
println("");         // 手动换行
```

## 打印输出

### print / println

`print` 和 `println` 接受任意数量参数,用空格分隔:

```link
print("a", "b", "c");          // a b c (不换行)
println("a", "b", "c");        // a b c (换行)
println(1, 2, 3);              // 1 2 3
println("答案:", 42);          // 答案: 42
println("PI =", 3.14159);      // PI = 3.14159
```

### 打印列表

```link
let nums = [1, 2, 3];
println(nums);        // [1, 2, 3]

let nested = [[1, 2], [3, 4]];
println(nested);      // [[1, 2], [3, 4]]
```

## 综合示例

### 求列表最大值

```link
fn max_of(nums: list) -> i32 {
    let m = nums[0];
    for i in 1..len(nums) {
        if nums[i] > m {
            m = nums[i];
        }
    }
    m
}

let arr = [3, 1, 4, 1, 5, 9, 2, 6, 5];
println("最大值:", max_of(arr));   // 9
```

### 求和

```link
fn sum_of(nums: list) -> i32 {
    let total = 0;
    for i in 0..len(nums) {
        total = total + nums[i];
    }
    total
}

let arr = [1, 2, 3, 4, 5];
println("总和:", sum_of(arr));   // 15
```

### 反转字符串

```link
fn reverse(s: str) -> str {
    let n = len(s);
    let result = "";
    let i = n - 1;
    while i >= 0 {
        result = result + s[i];
        i = i - 1;
    }
    result
}

println(reverse("hello"));   // olleh
println(reverse("你好"));     // 好你
```

### 矩阵转置

```link
fn transpose(m: list) -> list {
    let rows = len(m);
    let cols = len(m[0]);
    let result = [];
    for i in 0..cols {
        let row = [];
        for j in 0..rows {
            row = row + [m[j][i]];
        }
        result = result + [row];
    }
    result
}

let m = [[1, 2, 3], [4, 5, 6]];
let t = transpose(m);
// t = [[1, 4], [2, 5], [3, 6]]
```

!!! note "v0.1 限制"
    上面示例用 `+` 拼接列表是临时方案,效率不高。后续版本会加入 `push` / `insert` 等方法。

## 下一步

- [多语言互联概述](../ffi/overview.md)
- [C 互连](../ffi/c.md)
