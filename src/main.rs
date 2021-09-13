use std::env;
use std::collections::VecDeque;

use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::OptimizationLevel;
use inkwell::module::Linkage;
use inkwell::values::{IntValue, PointerValue};
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;

struct WhileBlock<'ctx> {
    while_start: BasicBlock<'ctx>,
    while_body: BasicBlock<'ctx>,
    while_end: BasicBlock<'ctx>,
}

fn main() {
    // command line arguments
    let args: Vec<String> = env::args().collect();
    let program = args[1].clone();

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let i64_type = context.i64_type();
    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    // function declaration
    // declare i8* @calloc(i64)
    let calloc_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let calloc_func = module.add_function("calloc", calloc_type, Some(Linkage::External));
    // declare i32 @putchar(i32)
    let putchar_type = i32_type.fn_type(&[i32_type.into()], false);
    let putchar_func = module.add_function("putchar", putchar_type, None);
    // declare i32 @getchar()
    let getchar_type = i32_type.fn_type(&[], false);
    let getchar_func =module.add_function("getchar", getchar_type, Some(Linkage::External));
    // declare i32 @printf(i8*, ...)
    let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let printf_func = module.add_function("printf", printf_type, None);

    // define i32 @main() {
    let main_type = i32_type.fn_type(&[], false);
    let main_func = module.add_function("main", main_type, None);

    // mainにentryを作ってbuilderの位置を変更する
    let basic_block = context.append_basic_block(main_func, "entry");
    builder.position_at_end(basic_block);

    // ptr
    let buff = builder.build_alloca(i8_ptr_type, "buff");
    let ptr = builder.build_alloca(i8_ptr_type, "ptr");

    let calloc_call = builder.build_call(calloc_func, &[i64_type.const_int(30000, false).into(), i64_type.const_int(1, false).into()], "");
    builder.build_store(buff, calloc_call.try_as_basic_value().left().unwrap());

    builder.build_store(ptr, builder.build_load(buff, ""));

    // []block stack
    let mut while_blocks = VecDeque::new();

    for c in program.chars() {
        match c {
            '+' => {
                let ptr_load = builder.build_load(ptr, "").into_pointer_value();
                let ptr_load_load = builder.build_load(ptr_load, "").into_int_value();
                let res = builder.build_int_add(ptr_load_load, i8_type.const_int(1 as u64, false), "");
                builder.build_store(ptr_load, res);
            },
            '-' => {
                let ptr_load = builder.build_load(ptr, "").into_pointer_value();
                let ptr_load_load = builder.build_load(ptr_load, "").into_int_value();
                let res = builder.build_int_sub(ptr_load_load, i8_type.const_int(1 as u64, false), "");
                builder.build_store(ptr_load, res);
            },
            '>' => {
                let p = builder.build_load(ptr, "").into_pointer_value();
                let res = unsafe {
                    builder.build_in_bounds_gep(p, &[i32_type.const_int(1, false)], "")
                };
                builder.build_store(ptr, res);
            },
            '<' => {
                let p = builder.build_load(ptr, "").into_pointer_value();
                let res = unsafe {
                    // u64::MAXで-1が表現できるのはなぜ??
                    builder.build_in_bounds_gep(p, &[i32_type.const_int(u64::MAX as u64, true)], "")
                };
                builder.build_store(ptr, res);
            },
            '.' => {
                let val = builder.build_load(builder.build_load(ptr, "").into_pointer_value(), "").into_int_value();
                let sext = builder.build_int_s_extend(val, i32_type, "");
                builder.build_call(putchar_func, &[sext.into()], "");
            },
            '[' => {
                let while_block = WhileBlock {
                    while_start: context.append_basic_block(main_func, format!("start{}", while_blocks.len()).as_str()),
                    while_body: context.append_basic_block(main_func, format!("body{}", while_blocks.len()).as_str()),
                    while_end: context.append_basic_block(main_func, format!("end{}", while_blocks.len()).as_str()),
                };
                while_blocks.push_front(while_block);
                // moveが発生するのでcloneする
                let while_block = while_blocks.front().unwrap();

                builder.build_unconditional_branch(while_block.while_start);
                builder.position_at_end(while_block.while_start);

                let i8_zero = i8_type.const_int(0, false);
                let ptr_load = builder.build_load(ptr, "").into_pointer_value();
                let ptr_load_load = builder.build_load(ptr_load, "").into_int_value();
                let cmp = builder.build_int_compare(IntPredicate::NE, ptr_load_load, i8_zero, "");

                builder.build_conditional_branch(cmp, while_block.while_body, while_block.while_end);
                builder.position_at_end(while_block.while_body);
            },
            ']' => {
                if let Some(while_block) = while_blocks.pop_front() {
                    builder.build_unconditional_branch(while_block.while_start);
                    builder.position_at_end(while_block.while_end);
                } else {
                    panic!("unmatched ']'");
                };
            },
            _ => ()
        };
    }

    // ret i32 0
    builder.build_return(Some(&i32_type.const_int(0, false)));

    //module.print_to_stderr();

    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::Aggressive).unwrap();
    unsafe {
        execution_engine.get_function::<unsafe extern "C" fn()>("main").unwrap().call();
    }

}

