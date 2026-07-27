//! Python FFI 模块
//! 通过 libloading 动态加载 libpython,实现 Link 调用 Python 函数

use libloading::{Library, Symbol};
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::ptr;

use crate::Value;
use linkc_parser::TypeAnnotation;

/// Python 对象指针类型
pub type PyObject = *mut c_void;

/// Python 运行时,持有 libpython 库引用
pub struct PythonRuntime {
    lib: Library,
    initialized: bool,
}

impl std::fmt::Debug for PythonRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonRuntime")
            .field("initialized", &self.initialized)
            .finish()
    }
}

impl PythonRuntime {
    /// 自动查找并加载 libpython 共享库
    pub fn new() -> Result<Self, String> {
        let lib = unsafe {
            #[cfg(windows)]
            {
                // Windows: 尝试常见 Python DLL 名称
                Library::new("python3.dll")
                    .or_else(|_| Library::new("python310.dll"))
                    .or_else(|_| Library::new("python311.dll"))
                    .or_else(|_| Library::new("python39.dll"))
                    .or_else(|_| Library::new("python38.dll"))
                    .map_err(|e| format!("Failed to load python3.dll: {}. Please ensure Python is installed.", e))?
            }
            #[cfg(unix)]
            {
                Library::new("libpython3.so")
                    .or_else(|_| Library::new("libpython3.10.so"))
                    .or_else(|_| Library::new("libpython3.11.so"))
                    .or_else(|_| Library::new("libpython3.so.1.0"))
                    .or_else(|_| Library::new("libpython3.dylib"))
                    .map_err(|e| format!("Failed to load libpython3: {}", e))?
            }
        };

        let mut rt = PythonRuntime { lib, initialized: false };
        rt.initialize()?;
        Ok(rt)
    }

    /// 初始化 Python 解释器(如果尚未初始化)
    fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }
        unsafe {
            // Py_IsInitialized()
            let is_init: Symbol<extern "C" fn() -> c_int> =
                self.lib.get(b"Py_IsInitialized\0").map_err(|e| format!("Py_IsInitialized not found: {}", e))?;
            if is_init() == 0 {
                // Py_Initialize()
                let py_init: Symbol<extern "C" fn()> =
                    self.lib.get(b"Py_Initialize\0").map_err(|e| format!("Py_Initialize not found: {}", e))?;
                py_init();
            }
        }
        self.initialized = true;
        Ok(())
    }

    /// 调用 Python 模块中的函数
    /// 等价于: module.func(args...)
    pub fn call_module_func(
        &self,
        module: &str,
        func: &str,
        args: &[Value],
        return_type: &TypeAnnotation,
    ) -> Result<Value, String> {
        unsafe {
            // 导入模块: PyImport_ImportModule(name)
            let module_cstr = CString::new(module)
                .map_err(|e| format!("Invalid module name: {}", e))?;
            let py_import: Symbol<extern "C" fn(*const c_char) -> PyObject> =
                self.lib.get(b"PyImport_ImportModule\0")
                    .map_err(|e| format!("PyImport_ImportModule not found: {}", e))?;
            let py_module = py_import(module_cstr.as_ptr());
            if py_module.is_null() {
                return Err(format!("Failed to import Python module '{}'", module));
            }

            // 获取函数属性: PyObject_GetAttrString(obj, name)
            let func_cstr = CString::new(func)
                .map_err(|e| format!("Invalid function name: {}", e))?;
            let py_getattr: Symbol<extern "C" fn(PyObject, *const c_char) -> PyObject> =
                self.lib.get(b"PyObject_GetAttrString\0")
                    .map_err(|e| format!("PyObject_GetAttrString not found: {}", e))?;
            let py_func = py_getattr(py_module, func_cstr.as_ptr());
            if py_func.is_null() {
                return Err(format!("Function '{}' not found in module '{}'", func, module));
            }

            // 构建 Python 参数元组
            let py_args = self.build_args_tuple(args)?;

            // 调用函数: PyObject_CallObject(func, args)
            let py_call: Symbol<extern "C" fn(PyObject, PyObject) -> PyObject> =
                self.lib.get(b"PyObject_CallObject\0")
                    .map_err(|e| format!("PyObject_CallObject not found: {}", e))?;
            let py_result = py_call(py_func, py_args);

            // 释放参数和函数引用
            let py_decref: Symbol<extern "C" fn(PyObject)> =
                self.lib.get(b"Py_DecRef\0")
                    .map_err(|e| format!("Py_DecRef not found: {}", e))?;
            py_decref(py_args);
            py_decref(py_func);
            py_decref(py_module);

            if py_result.is_null() {
                let err_msg = self.get_error_message()?;
                return Err(format!("Python call failed: {}", err_msg));
            }

            // 转换返回值
            let result = self.pyobject_to_value(py_result, return_type);

            // 释放结果引用
            py_decref(py_result);

            result
        }
    }

    /// 调用 Python 内置函数(不需要导入模块)
    /// 例如:len([1,2,3])  ->  python 内置 len
    pub fn call_builtin(
        &self,
        func: &str,
        args: &[Value],
        return_type: &TypeAnnotation,
    ) -> Result<Value, String> {
        unsafe {
            // 获取 __builtins__ 模块
            let builtins_cstr = CString::new("__builtins__").unwrap();
            let py_import: Symbol<extern "C" fn(*const c_char) -> PyObject> =
                self.lib.get(b"PyImport_ImportModule\0")
                    .map_err(|e| format!("PyImport_ImportModule not found: {}", e))?;
            let py_builtins = py_import(builtins_cstr.as_ptr());
            if py_builtins.is_null() {
                return Err("Failed to get __builtins__".to_string());
            }

            let func_cstr = CString::new(func)
                .map_err(|e| format!("Invalid function name: {}", e))?;
            let py_getattr: Symbol<extern "C" fn(PyObject, *const c_char) -> PyObject> =
                self.lib.get(b"PyObject_GetAttrString\0")
                    .map_err(|e| format!("PyObject_GetAttrString not found: {}", e))?;
            let py_func = py_getattr(py_builtins, func_cstr.as_ptr());

            let py_decref: Symbol<extern "C" fn(PyObject)> =
                self.lib.get(b"Py_DecRef\0").unwrap();

            if py_func.is_null() {
                py_decref(py_builtins);
                return Err(format!("Python builtin '{}' not found", func));
            }

            let py_args = self.build_args_tuple(args)?;

            let py_call: Symbol<extern "C" fn(PyObject, PyObject) -> PyObject> =
                self.lib.get(b"PyObject_CallObject\0")
                    .map_err(|e| format!("PyObject_CallObject not found: {}", e))?;
            let py_result = py_call(py_func, py_args);

            py_decref(py_args);
            py_decref(py_func);
            py_decref(py_builtins);

            if py_result.is_null() {
                let err_msg = self.get_error_message()?;
                return Err(format!("Python call failed: {}", err_msg));
            }

            let result = self.pyobject_to_value(py_result, return_type);
            py_decref(py_result);
            result
        }
    }

    /// 执行任意 Python 代码字符串,返回表达式的结果
    /// 等价于:eval(code)
    pub fn eval(&self, code: &str, return_type: &TypeAnnotation) -> Result<Value, String> {
        unsafe {
            let code_cstr = CString::new(code)
                .map_err(|e| format!("Invalid code: {}", e))?;

            // PyRun_String(code, Py_eval_input, globals, locals)
            // Py_eval_input = 258
            let py_run: Symbol<extern "C" fn(*const c_char, c_int, PyObject, PyObject) -> PyObject> =
                self.lib.get(b"PyRun_String\0")
                    .map_err(|e| format!("PyRun_String not found: {}", e))?;

            // 获取 __main__ 模块的字典作为 globals/locals
            let main_cstr = CString::new("__main__").unwrap();
            let py_import: Symbol<extern "C" fn(*const c_char) -> PyObject> =
                self.lib.get(b"PyImport_ImportModule\0").unwrap();
            let py_main = py_import(main_cstr.as_ptr());
            if py_main.is_null() {
                return Err("Failed to get __main__ module".to_string());
            }

            // 获取模块字典: PyModule_GetDict
            let py_getdict: Symbol<extern "C" fn(PyObject) -> PyObject> =
                self.lib.get(b"PyModule_GetDict\0")
                    .map_err(|e| format!("PyModule_GetDict not found: {}", e))?;
            let py_dict = py_getdict(py_main);

            let py_result = py_run(code_cstr.as_ptr(), 258, py_dict, py_dict);

            let py_decref: Symbol<extern "C" fn(PyObject)> =
                self.lib.get(b"Py_DecRef\0").unwrap();
            py_decref(py_main);

            if py_result.is_null() {
                let err_msg = self.get_error_message()?;
                return Err(format!("Python eval failed: {}", err_msg));
            }

            let result = self.pyobject_to_value(py_result, return_type);
            py_decref(py_result);
            result
        }
    }

    /// 构建 Python 参数元组
    unsafe fn build_args_tuple(&self, args: &[Value]) -> Result<PyObject, String> {
        // PyTuple_New(size)
        let py_tuple_new: Symbol<extern "C" fn(c_long) -> PyObject> =
            self.lib.get(b"PyTuple_New\0")
                .map_err(|e| format!("PyTuple_New not found: {}", e))?;
        let py_tuple = py_tuple_new(args.len() as c_long);
        if py_tuple.is_null() {
            return Err("Failed to create Python tuple".to_string());
        }

        let py_setitem: Symbol<extern "C" fn(PyObject, c_long, PyObject) -> c_int> =
            self.lib.get(b"PyTuple_SetItem\0")
                .map_err(|e| format!("PyTuple_SetItem not found: {}", e))?;

        for (i, arg) in args.iter().enumerate() {
            let py_arg = self.value_to_pyobject(arg)?;
            // PyTuple_SetItem 会 steal reference,所以不需要额外 DECREF
            py_setitem(py_tuple, i as c_long, py_arg);
        }

        Ok(py_tuple)
    }

    /// Link Value → Python PyObject
    unsafe fn value_to_pyobject(&self, val: &Value) -> Result<PyObject, String> {
        match val {
            Value::Int(n) => {
                // PyLong_FromLongLong
                let py_long_from_ll: Symbol<extern "C" fn(i64) -> PyObject> =
                    self.lib.get(b"PyLong_FromLongLong\0")
                        .map_err(|e| format!("PyLong_FromLongLong not found: {}", e))?;
                Ok(py_long_from_ll(*n))
            }
            Value::Float(f) => {
                // PyFloat_FromDouble
                let py_float_from_d: Symbol<extern "C" fn(f64) -> PyObject> =
                    self.lib.get(b"PyFloat_FromDouble\0")
                        .map_err(|e| format!("PyFloat_FromDouble not found: {}", e))?;
                Ok(py_float_from_d(*f))
            }
            Value::Bool(b) => {
                // PyBool_FromLong
                let py_bool_from_l: Symbol<extern "C" fn(c_long) -> PyObject> =
                    self.lib.get(b"PyBool_FromLong\0")
                        .map_err(|e| format!("PyBool_FromLong not found: {}", e))?;
                Ok(py_bool_from_l(if *b { 1 } else { 0 }))
            }
            Value::Str(s) => {
                // PyUnicode_FromString
                let cstr = CString::new(s.as_str())
                    .map_err(|e| format!("Invalid string: {}", e))?;
                let py_unicode_from_str: Symbol<extern "C" fn(*const c_char) -> PyObject> =
                    self.lib.get(b"PyUnicode_FromString\0")
                        .map_err(|e| format!("PyUnicode_FromString not found: {}", e))?;
                Ok(py_unicode_from_str(cstr.as_ptr()))
            }
            Value::None => {
                // 使用 Py_BuildValue("z", NULL) 返回 None,更安全
                let py_buildvalue: Symbol<extern "C" fn(*const c_char, ...) -> PyObject> =
                    self.lib.get(b"Py_BuildValue\0")
                        .map_err(|e| format!("Py_BuildValue not found: {}", e))?;
                let fmt = CString::new("z").unwrap();
                Ok(py_buildvalue(fmt.as_ptr(), ptr::null::<c_void>()))
            }
            Value::List(items) => {
                // PyList_New(size)
                let py_list_new: Symbol<extern "C" fn(c_long) -> PyObject> =
                    self.lib.get(b"PyList_New\0")
                        .map_err(|e| format!("PyList_New not found: {}", e))?;
                let py_list = py_list_new(items.len() as c_long);
                if py_list.is_null() {
                    return Err("Failed to create Python list".to_string());
                }
                let py_setitem: Symbol<extern "C" fn(PyObject, c_long, PyObject) -> c_int> =
                    self.lib.get(b"PyList_SetItem\0").unwrap();
                for (i, item) in items.iter().enumerate() {
                    let py_item = self.value_to_pyobject(item)?;
                    py_setitem(py_list, i as c_long, py_item);
                }
                Ok(py_list)
            }
            _ => Err(format!("Cannot convert Link {} to Python", val.type_name())),
        }
    }

    /// Python PyObject → Link Value
    unsafe fn pyobject_to_value(&self, obj: PyObject, type_hint: &TypeAnnotation) -> Result<Value, String> {
        // 检查 None: PyObject_RichCompare(obj, Py_None, Py_EQ)
        if self.is_none(obj)? {
            return Ok(Value::None);
        }

        match type_hint {
            TypeAnnotation::I32 | TypeAnnotation::I64 | TypeAnnotation::U32 | TypeAnnotation::U64 => {
                // PyLong_AsLongLong
                let py_long_as_ll: Symbol<extern "C" fn(PyObject) -> i64> =
                    self.lib.get(b"PyLong_AsLongLong\0")
                        .map_err(|e| format!("PyLong_AsLongLong not found: {}", e))?;
                Ok(Value::Int(py_long_as_ll(obj)))
            }
            TypeAnnotation::F32 | TypeAnnotation::F64 => {
                // PyFloat_AsDouble
                let py_float_as_d: Symbol<extern "C" fn(PyObject) -> f64> =
                    self.lib.get(b"PyFloat_AsDouble\0")
                        .map_err(|e| format!("PyFloat_AsDouble not found: {}", e))?;
                Ok(Value::Float(py_float_as_d(obj)))
            }
            TypeAnnotation::Bool => {
                // PyObject_IsTrue
                let py_is_true: Symbol<extern "C" fn(PyObject) -> c_int> =
                    self.lib.get(b"PyObject_IsTrue\0")
                        .map_err(|e| format!("PyObject_IsTrue not found: {}", e))?;
                Ok(Value::Bool(py_is_true(obj) != 0))
            }
            TypeAnnotation::Str => {
                // Windows 上 PyUnicode_AsUTF8 可能找不到,用 PyObject_Str + PyUnicode_AsUTF8AndSize
                // 或者用更兼容的 PyUnicode_EncodeFSDefault -> PyBytes_AsString
                let s = self.pyobject_to_string(obj)?;
                Ok(Value::Str(s))
            }
            _ => {
                // 未知类型,尝试通用转换
                self.pyobject_to_value_generic(obj)
            }
        }
    }

    /// 通用 PyObject → Value 转换(不依赖类型提示)
    unsafe fn pyobject_to_value_generic(&self, obj: PyObject) -> Result<Value, String> {
        // 检查 None
        if self.is_none(obj)? {
            return Ok(Value::None);
        }

        // 检查是否是 bool (要在 int 之前检查,因为 bool 是 int 的子类)
        let py_bool_check: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyBool_Check\0").map_err(|e| format!("{}", e))?;
        if py_bool_check(obj) != 0 {
            let py_is_true: Symbol<extern "C" fn(PyObject) -> c_int> =
                self.lib.get(b"PyObject_IsTrue\0").unwrap();
            return Ok(Value::Bool(py_is_true(obj) != 0));
        }

        // 检查是否是 int
        let py_long_check: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyLong_Check\0").map_err(|e| format!("{}", e))?;
        if py_long_check(obj) != 0 {
            let py_long_as_ll: Symbol<extern "C" fn(PyObject) -> i64> =
                self.lib.get(b"PyLong_AsLongLong\0").unwrap();
            return Ok(Value::Int(py_long_as_ll(obj)));
        }

        // 检查是否是 float
        let py_float_check: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyFloat_Check\0").map_err(|e| format!("{}", e))?;
        if py_float_check(obj) != 0 {
            let py_float_as_d: Symbol<extern "C" fn(PyObject) -> f64> =
                self.lib.get(b"PyFloat_AsDouble\0").unwrap();
            return Ok(Value::Float(py_float_as_d(obj)));
        }

        // 检查是否是 str
        let py_unicode_check: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyUnicode_Check\0").map_err(|e| format!("{}", e))?;
        if py_unicode_check(obj) != 0 {
            let s = self.pyobject_to_string(obj)?;
            return Ok(Value::Str(s));
        }

        // 检查是否是 list
        let py_list_check: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyList_Check\0").map_err(|e| format!("{}", e))?;
        if py_list_check(obj) != 0 {
            let py_list_size: Symbol<extern "C" fn(PyObject) -> c_long> =
                self.lib.get(b"PyList_Size\0").unwrap();
            let py_list_getitem: Symbol<extern "C" fn(PyObject, c_long) -> PyObject> =
                self.lib.get(b"PyList_GetItem\0").unwrap();
            let size = py_list_size(obj);
            let mut items = Vec::new();
            for i in 0..size {
                let item = py_list_getitem(obj, i);
                // GetItem 返回 borrowed reference,不需要 DECREF
                items.push(self.pyobject_to_value_generic(item)?);
            }
            return Ok(Value::List(items));
        }

        Err(format!("Cannot convert Python object to Link value (generic)"))
    }

    /// 检查 PyObject 是否是 None
    unsafe fn is_none(&self, obj: PyObject) -> Result<bool, String> {
        // 使用 PyObject_IsTrue 比较 obj == None
        let py_buildvalue: Symbol<extern "C" fn(*const c_char, ...) -> PyObject> =
            self.lib.get(b"Py_BuildValue\0")
                .map_err(|e| format!("Py_BuildValue not found: {}", e))?;
        let fmt = CString::new("z").unwrap();
        let none_obj = py_buildvalue(fmt.as_ptr(), ptr::null::<c_void>());

        let py_richcompare: Symbol<extern "C" fn(PyObject, PyObject, c_int) -> PyObject> =
            self.lib.get(b"PyObject_RichCompare\0")
                .map_err(|e| format!("PyObject_RichCompare not found: {}", e))?;
        let result_obj = py_richcompare(obj, none_obj, 2);

        let py_is_true: Symbol<extern "C" fn(PyObject) -> c_int> =
            self.lib.get(b"PyObject_IsTrue\0").unwrap();
        let is_none = py_is_true(result_obj) != 0;

        let py_decref: Symbol<extern "C" fn(PyObject)> =
            self.lib.get(b"Py_DecRef\0").unwrap();
        py_decref(result_obj);
        py_decref(none_obj);

        Ok(is_none)
    }

    /// 通用字符串提取(兼容 Windows / Linux / macOS)
    /// 使用 PyUnicode_EncodeFSDefault 转为 bytes,再用 PyBytes_AsString 提取
    unsafe fn pyobject_to_string(&self, obj: PyObject) -> Result<String, String> {
        // 方法 1:直接尝试 PyUnicode_AsUTF8AndSize(返回 const char*,不需要 size)
        if let Ok(sym) = self.lib.get::<extern "C" fn(PyObject, *mut c_long) -> *const c_char>(b"PyUnicode_AsUTF8AndSize\0") {
            let mut size: c_long = 0;
            let ptr = sym(obj, &mut size);
            if !ptr.is_null() && size > 0 {
                let bytes = std::slice::from_raw_parts(ptr as *const u8, size as usize);
                return Ok(String::from_utf8_lossy(bytes).to_string());
            }
        }

        // 方法 2:用 PyUnicode_EncodeFSDefault 编码为 bytes,再 PyBytes_AsString
        let py_encode_fs: Symbol<extern "C" fn(PyObject) -> PyObject> =
            self.lib.get(b"PyUnicode_EncodeFSDefault\0")
                .map_err(|e| format!("PyUnicode_EncodeFSDefault not found: {}", e))?;
        let bytes_obj = py_encode_fs(obj);
        if bytes_obj.is_null() {
            return Err("Failed to encode Python string".to_string());
        }

        let py_bytes_as_string: Symbol<extern "C" fn(PyObject) -> *const c_char> =
            self.lib.get(b"PyBytes_AsString\0")
                .or_else(|_| self.lib.get(b"PyString_AsString\0"))
                .map_err(|e| format!("PyBytes_AsString not found: {}", e))?;
        let py_bytes_size: Symbol<extern "C" fn(PyObject) -> c_long> =
            self.lib.get(b"PyBytes_Size\0")
                .or_else(|_| self.lib.get(b"PyString_Size\0"))
                .map_err(|e| format!("PyBytes_Size not found: {}", e))?;

        let ptr = py_bytes_as_string(bytes_obj);
        let size = py_bytes_size(bytes_obj);

        let py_decref: Symbol<extern "C" fn(PyObject)> =
            self.lib.get(b"Py_DecRef\0").unwrap();
        py_decref(bytes_obj);

        if ptr.is_null() || size <= 0 {
            return Err("Failed to extract bytes from Python string".to_string());
        }

        let bytes = std::slice::from_raw_parts(ptr as *const u8, size as usize);
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    /// 获取 Python 异常信息
    unsafe fn get_error_message(&self) -> Result<String, String> {
        // PyErr_Fetch(&type, &value, &traceback)
        let mut err_type: PyObject = ptr::null_mut();
        let mut err_value: PyObject = ptr::null_mut();
        let mut err_tb: PyObject = ptr::null_mut();

        let py_err_fetch: Symbol<extern "C" fn(*mut PyObject, *mut PyObject, *mut PyObject)> =
            self.lib.get(b"PyErr_Fetch\0")
                .map_err(|e| format!("PyErr_Fetch not found: {}", e))?;
        py_err_fetch(&mut err_type, &mut err_value, &mut err_tb);

        if err_value.is_null() {
            return Ok("Unknown Python error".to_string());
        }

        let py_str: Symbol<extern "C" fn(PyObject) -> PyObject> =
            self.lib.get(b"PyObject_Str\0")
                .map_err(|e| format!("PyObject_Str not found: {}", e))?;
        let str_obj = py_str(err_value);

        let ptr = if let Ok(sym) = self.lib.get::<extern "C" fn(PyObject) -> *const c_char>(b"PyUnicode_AsUTF8\0") {
            sym(str_obj)
        } else if let Ok(sym) = self.lib.get::<extern "C" fn(PyObject) -> *const c_char>(b"_PyUnicode_AsString\0") {
            sym(str_obj)
        } else {
            let py_decref: Symbol<extern "C" fn(PyObject)> = self.lib.get(b"Py_DecRef\0").unwrap();
            py_decref(str_obj);
            return Ok("Unknown Python error (cannot decode)".to_string());
        };

        let msg = if ptr.is_null() {
            "Unknown Python error".to_string()
        } else {
            std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string()
        };

        let py_decref: Symbol<extern "C" fn(PyObject)> =
            self.lib.get(b"Py_DecRef\0").unwrap();
        if !str_obj.is_null() { py_decref(str_obj); }
        if !err_value.is_null() { py_decref(err_value); }
        if !err_type.is_null() { py_decref(err_type); }
        if !err_tb.is_null() { py_decref(err_tb); }

        Ok(msg)
    }
}
