#[cfg(windows)]
pub fn contain(
    child: &tokio::process::Child,
) -> crate::error::AppResult<std::os::windows::io::OwnedHandle> {
    use std::os::windows::io::FromRawHandle;
    use windows::Win32::{
        Foundation::HANDLE,
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
            SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        },
    };
    let failure = |error: windows::core::Error| {
        crate::error::AppError::Internal(format!(
            "Não foi possível proteger o processo do servidor: {error}"
        ))
    };
    unsafe {
        let job = CreateJobObjectW(None, windows::core::PCWSTR::null()).map_err(failure)?;
        let owned = std::os::windows::io::OwnedHandle::from_raw_handle(job.0);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of_val(&limits) as u32,
        )
        .map_err(failure)?;
        let handle = child.raw_handle().ok_or_else(|| {
            crate::error::AppError::InvalidState("O processo do servidor já encerrou.".into())
        })?;
        AssignProcessToJobObject(job, HANDLE(handle)).map_err(failure)?;
        Ok(owned)
    }
}

#[cfg(target_os = "linux")]
pub fn prepare(command: &mut tokio::process::Command) {
    let parent = std::process::id();
    unsafe {
        command.pre_exec(move || {
            unsafe extern "C" {
                fn prctl(option: i32, value: usize, arg3: usize, arg4: usize, arg5: usize) -> i32;
                fn getppid() -> i32;
            }
            if prctl(1, 15, 0, 0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if getppid() as u32 != parent {
                return Err(std::io::Error::from_raw_os_error(125));
            }
            Ok(())
        });
    }
}
