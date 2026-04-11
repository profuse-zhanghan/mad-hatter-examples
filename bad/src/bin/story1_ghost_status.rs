// Story 1: Ghost Status Code (幽灵状态码)
//
// 场景：用户管理系统，从DB读取状态字符串，做权限判断。
// 三个隐患：
//   1. 字面量散落，大小写不一致 → 运行时静默失配
//   2. .to_string() 类型擦除 → 本该是enum的状态退化为String
//   3. if-else字符串比较无穷尽 → 新增状态编译器不提醒

/// 模拟从数据库读取用户状态
fn db_get_user_status() -> String {
    "active".to_string()
}

/// 根据状态判断权限
fn can_access(status: &str) -> bool {
    if status == "Active" {       // Bug: DB存"active"，这里写"Active"
        true
    } else if status == "suspended" {
        false
    } else {
        false                     // "inactive"被静默归入此处
    }
}

/// 模拟写回状态
fn db_set_status(new_status: &str) {
    println!("[DB] SET status={}", new_status);
}

fn main() {
    let status = db_get_user_status();

    if can_access(&status) {
        db_set_status("active");    // 又一处字面量
    } else {
        db_set_status("locked");    // 引入新状态，但can_access不知道它
    }
}
