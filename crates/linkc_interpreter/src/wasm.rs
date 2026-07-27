use std::collections::HashMap;
use wasmtime::{Engine, Module, Instance, Store, ValType};
use crate::Value;

#[derive(Default)]
pub struct WasmRuntime {
    engine: Engine,
    modules: HashMap<String, Module>,
    instances: HashMap<String, (Instance, Store<()>)>,
}

impl std::fmt::Debug for WasmRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmRuntime")
            .field("modules", &self.modules.keys().collect::<Vec<_>>())
            .field("instances", &self.instances.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            engine: Engine::default(),
            modules: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    pub fn load_module(&mut self, name: &str, path: &str) -> Result<(), String> {
        let module = Module::from_file(&self.engine, path)
            .map_err(|e| format!("Failed to load WASM module '{}': {}", path, e))?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    pub fn load_module_from_bytes(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let module = Module::new(&self.engine, bytes)
            .map_err(|e| format!("Failed to compile WASM module '{}': {}", name, e))?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    pub fn instantiate_module(&mut self, name: &str) -> Result<(), String> {
        if self.instances.contains_key(name) {
            return Ok(());
        }
        
        let module = self.modules.get(name)
            .ok_or_else(|| format!("Module '{}' not loaded", name))?.clone();
        
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| format!("Failed to instantiate module '{}': {}", name, e))?;
        
        self.instances.insert(name.to_string(), (instance, store));
        Ok(())
    }

    pub fn call_func(
        &mut self,
        module_name: &str,
        func_name: &str,
        args: &[Value],
        ret_type: Option<&linkc_parser::TypeAnnotation>,
    ) -> Result<Value, String> {
        if !self.instances.contains_key(module_name) {
            self.instantiate_module(module_name)?;
        }

        let (instance, store) = self.instances.get_mut(module_name)
            .ok_or_else(|| format!("Instance for '{}' not found", module_name))?;

        let func = instance.get_func(&mut *store, func_name)
            .ok_or_else(|| format!("Function '{}' not found in module '{}'", func_name, module_name))?;

        let func_type = func.ty(&mut *store);
        let params_types = func_type.params().collect::<Vec<_>>();
        let results_types = func_type.results().collect::<Vec<_>>();

        let wasm_args = args.iter().zip(params_types.iter())
            .map(|(arg, ty)| value_to_wasm_val(arg, ty))
            .collect::<Result<Vec<_>, _>>()?;

        let num_results = results_types.len();
        let mut results = vec![wasmtime::Val::I32(0); num_results];

        func.call(&mut *store, &wasm_args, &mut results)
            .map_err(|e| format!("Error calling WASM function '{}': {}", func_name, e))?;

        if results.is_empty() {
            Ok(Value::None)
        } else {
            wasm_val_to_value(&results[0], ret_type)
        }
    }
}

fn value_to_wasm_val(value: &Value, ty: &ValType) -> Result<wasmtime::Val, String> {
    match (value, ty) {
        (Value::Int(i), ValType::I32) => Ok(wasmtime::Val::I32(*i as i32)),
        (Value::Int(i), ValType::I64) => Ok(wasmtime::Val::I64(*i)),
        (Value::Float(f), ValType::F32) => Ok(wasmtime::Val::F32((*f as f32).to_bits())),
        (Value::Float(f), ValType::F64) => Ok(wasmtime::Val::F64(f.to_bits())),
        (Value::Bool(b), ValType::I32) => Ok(wasmtime::Val::I32(if *b { 1 } else { 0 })),
        (v, t) => Err(format!("Cannot convert {} to WASM type {:?}", v.type_name(), t)),
    }
}

fn wasm_val_to_value(val: &wasmtime::Val, expected_type: Option<&linkc_parser::TypeAnnotation>) -> Result<Value, String> {
    match (val, expected_type) {
        (wasmtime::Val::I32(i), Some(linkc_parser::TypeAnnotation::Bool)) => {
            Ok(Value::Bool(*i != 0))
        }
        (wasmtime::Val::I32(i), _) => Ok(Value::Int(*i as i64)),
        (wasmtime::Val::I64(i), _) => Ok(Value::Int(*i)),
        (wasmtime::Val::F32(bits), _) => Ok(Value::Float(f32::from_bits(*bits) as f64)),
        (wasmtime::Val::F64(bits), _) => Ok(Value::Float(f64::from_bits(*bits))),
        _ => Err(format!("Cannot convert WASM value {:?} to Link Value", val)),
    }
}
