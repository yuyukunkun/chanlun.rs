use chanlun::business::observer::观察者;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: {} <.nb文件路径>", args[0]);
        std::process::exit(1);
    }

    let 文件路径 = &args[1];
    println!("读取文件: {}", 文件路径);

    match 观察者::读取数据文件(文件路径, None) {
        Ok(观察员) => {
            println!("符号: {}", 观察员.符号);
            println!("周期: {}", 观察员.周期);
            println!("普K数量: {}", 观察员.普通K线序列.len());
            println!("缠K数量: {}", 观察员.缠论K线序列.len());
            println!("分型数量: {}", 观察员.分型序列.len());
            println!("笔数量: {}", 观察员.笔序列.len());
            println!("笔中枢数量: {}", 观察员.笔_中枢序列.len());
            println!("线段数量: {}", 观察员.线段序列.len());
            println!("中枢数量: {}", 观察员.中枢序列.len());

            println!("\n===== 数据分析 =====\n");
            观察员.测试_保存数据(None);
        }
        Err(e) => {
            eprintln!("读取失败: {}", e);
            std::process::exit(1);
        }
    }
}
