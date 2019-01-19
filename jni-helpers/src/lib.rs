#[macro_use]
extern crate lazy_static;

extern crate jni_sys;

use std::collections::HashMap;
use std::any::Any;

use std::sync::Arc;
use std::sync::Weak;
use std::sync::RwLock;

use std::ptr;

use std::ops::Deref;

/*
pub trait ArcUpgradable<T: ?Sized>: Send + Sized + Sync
    where T: Sync + Send + Sized,
{
    fn upgrade(&self) -> Option<Arc<T>>;
}

impl<T: ?Sized> ArcUpgradable<T> for Arc<T>
    where T: Sync + Send + Sized
{
    fn upgrade(&self) -> Option<Arc<T>> {
        Option::Some(self.clone())
    }
}

impl<T: ?Sized> ArcUpgradable<T> for Weak<T>
    where T: Sync + Send + Sized
{
    fn upgrade(&self) -> Option<Arc<T>> {
        Weak::upgrade(&self)
    }
}
*/
#[derive(Debug)]
pub struct JavaObject<T: ?Sized>
    where T: LocallyCachedJavaClass + Sized + Sync + Any,
{
    item: RwLock<Option<Weak<T>>>,
    cchmthd: CacheMethod,
    jobj: std::ptr::NonNull<jni_sys::_jobject>,
    jvm: std::ptr::NonNull<jni_sys::JNIInvokeInterface_>,
    // _phantom: std::marker::PhantomData<T>,
}

unsafe impl<T> std::marker::Send for JavaObject<T>
    where T: LocallyCachedJavaClass { }
unsafe impl<T> std::marker::Sync for JavaObject<T>
    where T: LocallyCachedJavaClass { }

impl<T: ?Sized> JavaObject<T>
where 
    T: LocallyCachedJavaClass + Sync,
{
    pub fn java_object(&self) -> &std::ptr::NonNull<jni_sys::_jobject> {
        &self.jobj
    }
    pub fn java_vm(&self) -> &std::ptr::NonNull<jni_sys::JNIInvokeInterface_> {
        &self.jvm
    }
    pub fn cache_method(&self) -> &CacheMethod {
        &self.cchmthd
    }
    pub fn with_item<R, F>(&self, f: F) -> Result<R, CacheItemErr>
        where F: FnOnce(Arc<T>) -> R
    {
        let unw = self.item.read().unwrap();
        match std::sync::RwLockReadGuard::deref(&unw) {
            Option::None => Err(CacheItemErr::NoInit),
            Option::Some(x) => match x.upgrade() {
                Option::None => Err(CacheItemErr::Dropped),
                Option::Some(y) => Ok(f(y)),
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CacheItemErr {
    /// Item is not initialized
    NoInit,
    Dropped,
}


#[derive(Hash, PartialEq, Eq, Debug)]
pub enum CacheMethod {
    /// No resuse will be attempted, calls from java will always first create a new struct.
    Avoid,
    /// Reuse as long as rust has not dropped the cache item.
    Reuse,
}

pub trait JavaClass {
    fn java_class_name() -> &'static str;
    fn new<'a>(env: &'a jni_sys::JNIEnv, obj: &'a jni_sys::jobject) -> Self;
}

pub trait LocallyCachedJavaClass : JavaClass + Any + Send + Sized
    where Self: Sized + Sync + Any + Send
{
    fn cache_method() -> CacheMethod {
        CacheMethod::Reuse
    }
    /// 
    fn new<'a>(env: &'a jni_sys::JNIEnv, obj: Arc<JavaObject<Self>>) -> Self
        where
            Self: Sized,
    {
        let j = obj.jobj;
        <Self as JavaClass>::new(env, &j.as_ptr())
    }
}

lazy_static! {
    static ref LOCAL_CACHE: RwLock<HashMap<&'static str, HashMap<u64, Weak<Any + Sync + Send>>>> = RwLock::new(HashMap::new()); //createHashMap();
    // static ref LOCAL_CACHE: RwLock<HashMap<&'static str, HashMap<u64, Weak<Any + Sync + Send>>>> = RwLock::new(HashMap::new());
    // static ref LOCAL_CACHE: RwLock<HashMap<&'static str, HashMap<u64, Weak<RwLock<JavaObject<LocallyCachedJavaClass + std::marker::Sized, Arc<LocallyCachedJavaClass + std::marker::Sized>>>>>>> = RwLock::new(HashMap::new());
    // static ref LOCAL_CACHE: RwLock<HashMap<&'static str, HashMap<u64, Weak<JavaObject<LocallyCachedJavaClass + std::marker::Sized, Arc<LocallyCachedJavaClass + Sized>>>>>> = RwLock::new(HashMap::new());
}

/// must run on attached thread
pub fn cache_java_object<T>(clss: &str, env: jni_sys::JNIEnv, obj: jni_sys::jobject) -> Result<Arc<T>, String> 
    where T: LocallyCachedJavaClass + Any + Sized + Send,
{
    let m = T::cache_method();

    let reuse = match m {
        CacheMethod::Reuse => { 
            let r = LOCAL_CACHE.read();
            let u = r.unwrap();
            match u.get(clss) {
                Option::None => true,
                Option::Some(x) => match x.get(&(env as u64)) {
                    Option::None => true,
                    Option::Some(o) => match o.upgrade() {
                        Option::None => true,
                        Option::Some(p) => match p.downcast::<T>() {
                            Result::Err(_) => return Result::Err("Unable to downcast".to_string()),
                            Result::Ok(v) => return Result::Ok(v),
                        },
                    },
                },
            }
        },
        _ => false,
    };

    // do pointer stuff
    let jvm = unsafe {
        match (*env).GetJavaVM {
            Option::None => return Err("Could not get implementation of JNIEnv::GetJavaVM".to_string()),
            Option::Some(get_java_vm) => {
                let penv: *mut jni_sys::JNIEnv = ptr::null_mut();
                *penv = &*env;
                let pinv = ptr::null_mut();
                let _r = get_java_vm(penv, pinv);
                // TODO: check return value `r`
                if pinv.is_null() {
                    return Err("Empty result #0 when calling JNIEnv::GetJavaVM".to_string());
                }
                let p = *pinv;
                if p.is_null() {
                    return Err("Empty result #1 when calling JNIEnv::GetJavaVM".to_string());
                }
                (*p) as *mut jni_sys::JNIInvokeInterface_
            },
        }
    };

    // create item
    let o = unsafe { Arc::new(JavaObject {
        item: RwLock::new(Option::None),
        cchmthd: m,
        jobj: std::ptr::NonNull::new_unchecked(obj),
        jvm: std::ptr::NonNull::new_unchecked(jvm),
        // _phantom: std::marker::PhantomData
    }) };

    let mut w: std::sync::RwLockWriteGuard<Option<Weak<T>>> = (*o).item.write().unwrap();

    // create struct and store in cached object
    let inner = Arc::new(LocallyCachedJavaClass::new(&env, Arc::clone(&o)));
    *w = Option::Some(Arc::downgrade(&inner.clone()));
    std::mem::drop(w);

    if reuse {
        let mut wr = LOCAL_CACHE.write().unwrap();
        match wr.get_mut(clss) { 
            Option::None => (),
            Option::Some(x) => {
                let v = (o as Arc<Any + std::marker::Send + std::marker::Sync>).clone();
                x.insert(env as u64, Arc::downgrade(&v));
            },
        };
    }

    Ok(inner)
}
