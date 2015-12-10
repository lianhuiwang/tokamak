macro_rules! llvm_ret(
  ($ret:expr, $out:expr, $err:expr) => (
    if $ret == 0 {
      Ok($out)
    } else {
      Err(chars_to_str($err).to_string())
    }
  );
);

macro_rules! expect_noerr(
  ($ret:expr, $message:expr) => (
    if $ret == 1 {
      return Err($message.to_string());
    }
  );  
);

macro_rules! impl_dispose (
  ($ty:ty, $func:expr) => (
    impl Drop for $ty {
      fn drop(&mut self) {
        unsafe {
          $func(self.0);
        }
      }
    }    
  );
);