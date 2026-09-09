import re

with open('src-tauri/src/commands/launch.rs', 'r') as f:
    code = f.read()

code = code.replace(
    '''let pid = launcher
        .launch(
            &detail,
            &account.username,
            &final_uuid,
            &final_token,
            user_type,
            &game_dir,
        )
        .await?;''',
    '''let pid = launcher
        .launch(
            &detail,
            &account.username,
            &final_uuid,
            &final_token,
            user_type,
            &game_dir,
            &profile,
        )
        .await?;'''
)

with open('src-tauri/src/commands/launch.rs', 'w') as f:
    f.write(code)
