#[cfg(target_os = "windows")]
mod native {
    use std::ffi::c_void;
    type Hwnd = *mut c_void;
    #[repr(C)] struct Point { x:i32, y:i32 }
    #[link(name="user32")]
    unsafe extern "system" {
        fn EnumWindows(callback:unsafe extern "system" fn(Hwnd,isize)->i32,param:isize)->i32;
        fn FindWindowExW(parent:Hwnd,after:Hwnd,class:*const u16,title:*const u16)->Hwnd;
        fn GetWindowLongPtrW(hwnd:Hwnd,index:i32)->isize;
        fn SetWindowLongPtrW(hwnd:Hwnd,index:i32,value:isize)->isize;
        fn SetParent(hwnd:Hwnd,parent:Hwnd)->Hwnd;
        fn GetParent(hwnd:Hwnd)->Hwnd;
        fn IsWindow(hwnd:Hwnd)->i32;
        fn MapWindowPoints(from:Hwnd,to:Hwnd,points:*mut Point,count:u32)->i32;
        fn SetWindowPos(hwnd:Hwnd,after:Hwnd,x:i32,y:i32,w:i32,h:i32,flags:u32)->i32;
    }
    unsafe extern "system" fn find_host(hwnd:Hwnd,result:isize)->i32 {
        let class:Vec<u16>="SHELLDLL_DefView\0".encode_utf16().collect();
        if !unsafe{FindWindowExW(hwnd,std::ptr::null_mut(),class.as_ptr(),std::ptr::null())}.is_null(){
            unsafe{*(result as *mut Hwnd)=hwnd;} return 0;
        } 1
    }
    fn to_client(parent:Hwnd,x:i32,y:i32)->Point{
        let mut point=Point{x,y};
        unsafe{MapWindowPoints(std::ptr::null_mut(),parent,&mut point,1);}
        point
    }
    pub fn attach(handle:isize,x:i32,y:i32)->Result<(),String>{
        let hwnd=handle as Hwnd;let mut host:Hwnd=std::ptr::null_mut();
        unsafe{
            EnumWindows(find_host,(&mut host as *mut Hwnd) as isize);
            if host.is_null(){return Err("找不到桌面窗口，请从托盘恢复列表后重试".into());}
            let old=GetWindowLongPtrW(hwnd,-16);
            SetWindowLongPtrW(hwnd,-16,(old & !(0x8000_0000_u32 as isize)) | 0x4000_0000);
            SetParent(hwnd,host);
            if GetParent(hwnd)!=host {SetWindowLongPtrW(hwnd,-16,old);return Err("桌面挂载失败，请从托盘恢复列表后重试".into());}
            let pos=to_client(host,x,y);
            if SetWindowPos(hwnd,std::ptr::null_mut(),pos.x,pos.y,0,0,0x0001|0x0040|0x0020|0x0010)==0{return Err("桌面列表定位失败".into());}
        } Ok(())
    }
    pub fn attached(handle:isize)->bool {
        let hwnd=handle as Hwnd; unsafe{IsWindow(hwnd)!=0 && !GetParent(hwnd).is_null() && IsWindow(GetParent(hwnd))!=0}
    }
    pub fn detach(handle:isize)->Result<(),String>{
        let hwnd=handle as Hwnd;
        unsafe{
            if !attached(handle){return Ok(());}
            let style=GetWindowLongPtrW(hwnd,-16);
            SetParent(hwnd,std::ptr::null_mut());
            SetWindowLongPtrW(hwnd,-16,(style & !(0x4000_0000_u32 as isize))|0x0008_0000);
            let _=SetWindowPos(hwnd,std::ptr::null_mut(),0,0,0,0,0x0001|0x0002|0x0010|0x0020);
        }
        Ok(())
    }
    pub fn move_to(handle:isize,x:i32,y:i32)->Result<(),String>{
        let hwnd=handle as Hwnd;
        unsafe{
            let parent=GetParent(hwnd);if parent.is_null(){return Err("桌面挂载已失效，请重新显示列表".into());}
            let pos=to_client(parent,x,y);
            if SetWindowPos(hwnd,std::ptr::null_mut(),pos.x,pos.y,0,0,0x0001|0x0004|0x0010)==0 {return Err("移动列表失败".into());}
        } Ok(())
    }
}
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn attach(handle:isize,x:i32,y:i32)->Result<(),String>{native::attach(handle,x,y)}
#[cfg(target_os = "windows")]
pub fn attached(handle:isize)->bool{native::attached(handle)}
#[cfg(target_os = "windows")]
pub fn detach(handle:isize)->Result<(),String>{native::detach(handle)}
#[cfg(target_os = "windows")]
pub fn move_to(handle:isize,x:i32,y:i32)->Result<(),String>{native::move_to(handle,x,y)}
