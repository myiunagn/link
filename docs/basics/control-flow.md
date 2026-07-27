# 控制流

## if / else if / else

```link
let score = 85;

if score >= 90 {
    println("优秀");
} else if score >= 80 {
    println("良好");
} else if score >= 60 {
    println("及格");
} else {
    println("不及格");
}
```

### if 表达式

`if` 也可以作为表达式使用,必须有 `else` 分支:

```link
let x = 10;
let label = if x > 0 { "正数" } else { "非正" };
println(label);
```

!!! warning "类型一致"
    `if` 表达式的两个分支必须返回相同类型,否则报错。

## while 循环

```link
let i = 0;
while i < 5 {
    println("i =", i);
    i = i + 1;
}
```

### 配合 break / continue

```link
let i = 0;
while true {
    if i >= 10 { break; }
    i = i + 1;
    if i % 2 == 0 { continue; }
    println("奇数:", i);
}
```

## for 循环

`for` 是范围式循环,**左闭右开** `[start, end)`:

```link
// 0, 1, 2 (不含 3)
for i in 0..3 {
    println(i);
}

// 5, 6, 7, 8, 9
for n in 5..10 {
    println(n);
}
```

### 配合 break / continue

```link
for i in 0..100 {
    if i * i > 50 { break; }
    if i % 2 == 1 { continue; }
    println("偶数平方:", i * i);
}
```

## loop 无限循环

`loop` 等同于 `while true`,必须用 `break` 退出:

```link
let count = 0;
loop {
    println("count =", count);
    count = count + 1;
    if count >= 5 { break; }
}
```

## break

立即跳出最近的 `while` / `for` / `loop` 循环:

```link
for i in 0..10 {
    if i == 5 { break; }
    println(i);   // 打印 0,1,2,3,4
}
```

## continue

跳过本次循环体剩余部分,进入下一次迭代:

```link
for i in 0..5 {
    if i == 2 { continue; }
    println(i);   // 打印 0,1,3,4 (跳过 2)
}
```

## return

从函数返回。可带值,也可不带(返回 `none`):

```link
fn is_adult(age: i32) -> bool {
    return age >= 18;
}

fn greet() {
    println("hi");
    return;        // 等价于 return none;
}

fn early_exit(x: i32) -> i32 {
    if x < 0 { return -1; }
    if x == 0 { return 0; }
    x * 2
}
```

!!! note "隐式返回"
    函数最后一条表达式的值会作为隐式返回值。`return` 用于提前退出。

```link
fn double(x: i32) -> i32 {
    x * 2          // 隐式返回
}

fn explicit_double(x: i32) -> i32 {
    return x * 2;  // 显式返回
}
```

## 嵌套循环

```link
// 打印九九乘法表
for i in 1..10 {
    for j in 1..10 {
        print(i * j, " ");
    }
    println("");
}
```

### break 只跳出最近一层

```link
for i in 0..3 {
    for j in 0..3 {
        if j == 1 { break; }   // 只跳出内层
        println(i, j);
    }
}
```

!!! note "v0.1 限制"
    v0.1 暂不支持带标签的 `break 'outer` / `continue 'outer`。后续版本会加入。

## 综合示例

### FizzBuzz

```link
for n in 1..16 {
    if n % 15 == 0 {
        println("FizzBuzz");
    } else if n % 3 == 0 {
        println("Fizz");
    } else if n % 5 == 0 {
        println("Buzz");
    } else {
        println(n);
    }
}
```

### 二分查找

```link
fn binary_search(nums: list, target: i32) -> i32 {
    let lo = 0;
    let hi = len(nums);
    
    while lo < hi {
        let mid = (lo + hi) / 2;
        if nums[mid] == target {
            return mid;
        } else if nums[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    
    return -1;
}

let arr = [1, 3, 5, 7, 9, 11, 13];
let idx = binary_search(arr, 7);
println("找到索引:", idx);   // 3
```

## 下一步

- [函数](functions.md)
- [列表与字符串](collections.md)
