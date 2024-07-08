# Throttle Lock

Throttle is an activity counter that can be used to monitor
and limit activity such as incoming connections and sign in
attempts.

Disclaimer: This library is *not* guaranteed and is *not*
warranted to be fit for purpose. This code is not an
appropriate replacement for dedicated security software
and hardware. For demonstration purposes only.

## Examples

Limit calls to an API to 5 per second, or lockout for one minute

```
use throttle_lock::Throttle;

let mut counter = Throttle::new(1000, 5, 1000*60);
if counter.is_throttled() {
    println!("Try again later")
}
```

Limit signin attempts on an email address to 5 per minute, or
lockout for 5 minutes.

```
use throttle_lock::ThrottleHash;

let mut counter = ThrottleHash::new(60*1000, 5, 3*60*1000);
let email:String = "john@example.com".to_string();
if counter.is_throttled(&email) {
    println!("Try again later")
}
```
