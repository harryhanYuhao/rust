use expect_test::expect;

use super::{check_infer, check_no_mismatches, check_types};

#[test]
fn bug_484() {
    check_infer(
        r#"
        fn test() {
            let x = if true {};
        }
        "#,
        expect![[r#"
            10..37 '{     ... {}; }': ()
            20..21 'x': ()
            24..34 'if true {}': ()
            27..31 'true': bool
            32..34 '{}': ()
        "#]],
    );
}

#[test]
fn no_panic_on_field_of_enum() {
    check_infer(
        r#"
        enum X {}

        fn test(x: X) {
            x.some_field;
        }
        "#,
        expect![[r#"
            19..20 'x': X
            25..46 '{     ...eld; }': ()
            31..32 'x': X
            31..43 'x.some_field': {unknown}
        "#]],
    );
}

#[test]
fn bug_585() {
    check_infer(
        r#"
        fn test() {
            X {};
            match x {
                A::B {} => (),
                A::Y() => (),
            }
        }
        "#,
        expect![[r#"
            10..88 '{     ...   } }': ()
            16..20 'X {}': {unknown}
            26..86 'match ...     }': ()
            32..33 'x': {unknown}
            44..51 'A::B {}': {unknown}
            55..57 '()': ()
            67..73 'A::Y()': {unknown}
            77..79 '()': ()
        "#]],
    );
}

#[test]
fn bug_651() {
    check_infer(
        r#"
        fn quux() {
            let y = 92;
            1 + y;
        }
        "#,
        expect![[r#"
            10..40 '{     ...+ y; }': ()
            20..21 'y': i32
            24..26 '92': i32
            32..33 '1': i32
            32..37 '1 + y': i32
            36..37 'y': i32
        "#]],
    );
}

#[test]
fn recursive_vars() {
    check_infer(
        r#"
        fn test() {
            let y = unknown;
            [y, &y];
        }
        "#,
        expect![[r#"
            10..47 '{     ...&y]; }': ()
            20..21 'y': {unknown}
            24..31 'unknown': {unknown}
            37..44 '[y, &y]': [{unknown}; 2]
            38..39 'y': {unknown}
            41..43 '&y': &{unknown}
            42..43 'y': {unknown}
        "#]],
    );
}

#[test]
fn recursive_vars_2() {
    check_infer(
        r#"
        fn test() {
            let x = unknown;
            let y = unknown;
            [(x, y), (&y, &x)];
        }
        "#,
        expect![[r#"
            10..79 '{     ...x)]; }': ()
            20..21 'x': &{unknown}
            24..31 'unknown': &{unknown}
            41..42 'y': {unknown}
            45..52 'unknown': {unknown}
            58..76 '[(x, y..., &x)]': [(&{unknown}, {unknown}); 2]
            59..65 '(x, y)': (&{unknown}, {unknown})
            60..61 'x': &{unknown}
            63..64 'y': {unknown}
            67..75 '(&y, &x)': (&{unknown}, {unknown})
            68..70 '&y': &{unknown}
            69..70 'y': {unknown}
            72..74 '&x': &&{unknown}
            73..74 'x': &{unknown}
        "#]],
    );
}

#[test]
fn array_elements_expected_type() {
    check_no_mismatches(
        r#"
        fn test() {
            let x: [[u32; 2]; 2] = [[1, 2], [3, 4]];
        }
        "#,
    );
}

#[test]
fn infer_std_crash_1() {
    // caused stack overflow, taken from std
    check_infer(
        r#"
        enum Maybe<T> {
            Real(T),
            Fake,
        }

        fn write() {
            match something_unknown {
                Maybe::Real(ref mut something) => (),
            }
        }
        "#,
        expect![[r#"
            53..138 '{     ...   } }': ()
            59..136 'match ...     }': ()
            65..82 'someth...nknown': Maybe<{unknown}>
            93..123 'Maybe:...thing)': Maybe<{unknown}>
            105..122 'ref mu...ething': &mut {unknown}
            127..129 '()': ()
        "#]],
    );
}

#[test]
fn infer_std_crash_2() {
    // caused "equating two type variables, ...", taken from std
    check_infer(
        r#"
        fn test_line_buffer() {
            &[0, b'\n', 1, b'\n'];
        }
        "#,
        expect![[r#"
            22..52 '{     ...n']; }': ()
            28..49 '&[0, b...b'\n']': &[u8; 4]
            29..49 '[0, b'...b'\n']': [u8; 4]
            30..31 '0': u8
            33..38 'b'\n'': u8
            40..41 '1': u8
            43..48 'b'\n'': u8
        "#]],
    );
}

#[test]
fn infer_std_crash_3() {
    // taken from rustc
    check_infer(
        r#"
        pub fn compute() {
            match nope!() {
                SizeSkeleton::Pointer { non_zero: true, tail } => {}
            }
        }
        "#,
        expect![[r#"
            17..107 '{     ...   } }': ()
            23..105 'match ...     }': ()
            29..36 'nope!()': {unknown}
            47..93 'SizeSk...tail }': {unknown}
            81..85 'true': bool
            81..85 'true': bool
            87..91 'tail': {unknown}
            97..99 '{}': ()
        "#]],
    );
}

#[test]
fn infer_std_crash_4() {
    // taken from rustc
    check_infer(
        r#"
        pub fn primitive_type() {
            match *self {
                BorrowedRef { type_: Primitive(p), ..} => {},
            }
        }
        "#,
        expect![[r#"
            24..105 '{     ...   } }': ()
            30..103 'match ...     }': ()
            36..41 '*self': {unknown}
            37..41 'self': {unknown}
            52..90 'Borrow...), ..}': {unknown}
            73..85 'Primitive(p)': {unknown}
            83..84 'p': {unknown}
            94..96 '{}': ()
        "#]],
    );
}

#[test]
fn infer_std_crash_5() {
    // taken from rustc
    check_infer(
        r#"
        fn extra_compiler_flags() {
            for content in doesnt_matter {
                let name = if doesnt_matter {
                    first
                } else {
                    &content
                };

                let content = if ICE_REPORT_COMPILER_FLAGS_STRIP_VALUE.contains(&name) {
                    name
                } else {
                    content
                };
            }
        }
        "#,
        expect![[r#"
            26..322 '{     ...   } }': ()
            32..320 'for co...     }': ()
            36..43 'content': {unknown}
            47..60 'doesnt_matter': {unknown}
            61..320 '{     ...     }': ()
            75..79 'name': &{unknown}
            82..166 'if doe...     }': &{unknown}
            85..98 'doesnt_matter': bool
            99..128 '{     ...     }': &{unknown}
            113..118 'first': &{unknown}
            134..166 '{     ...     }': &{unknown}
            148..156 '&content': &{unknown}
            149..156 'content': {unknown}
            181..188 'content': &{unknown}
            191..313 'if ICE...     }': &{unknown}
            194..231 'ICE_RE..._VALUE': {unknown}
            194..247 'ICE_RE...&name)': bool
            241..246 '&name': &&{unknown}
            242..246 'name': &{unknown}
            248..276 '{     ...     }': &{unknown}
            262..266 'name': &{unknown}
            282..313 '{     ...     }': {unknown}
            296..303 'content': {unknown}
        "#]],
    );
}

#[test]
fn infer_nested_generics_crash() {
    // another crash found typechecking rustc
    check_infer(
        r#"
        struct Canonical<V> {
            value: V,
        }
        struct QueryResponse<V> {
            value: V,
        }
        fn test<R>(query_response: Canonical<QueryResponse<R>>) {
            &query_response.value;
        }
        "#,
        expect![[r#"
            91..105 'query_response': Canonical<QueryResponse<R>>
            136..166 '{     ...lue; }': ()
            142..163 '&query....value': &QueryResponse<R>
            143..157 'query_response': Canonical<QueryResponse<R>>
            143..163 'query_....value': QueryResponse<R>
        "#]],
    );
}

#[test]
fn infer_paren_macro_call() {
    check_infer(
        r#"
        macro_rules! bar { () => {0u32} }
        fn test() {
            let a = (bar!());
        }
        "#,
        expect![[r#"
            !0..4 '0u32': u32
            44..69 '{     ...()); }': ()
            54..55 'a': u32
        "#]],
    );
}

#[test]
fn infer_array_macro_call() {
    check_infer(
        r#"
        macro_rules! bar { () => {0u32} }
        fn test() {
            let a = [bar!()];
        }
        "#,
        expect![[r#"
            !0..4 '0u32': u32
            44..69 '{     ...()]; }': ()
            54..55 'a': [u32; 1]
            58..66 '[bar!()]': [u32; 1]
        "#]],
    );
}

#[test]
fn bug_1030() {
    check_infer(
        r#"
        struct HashSet<T, H>;
        struct FxHasher;
        type FxHashSet<T> = HashSet<T, FxHasher>;

        impl<T, H> HashSet<T, H> {
            fn default() -> HashSet<T, H> {}
        }

        pub fn main_loop() {
            FxHashSet::default();
        }
        "#,
        expect![[r#"
            143..145 '{}': ()
            168..197 '{     ...t(); }': ()
            174..192 'FxHash...efault': fn default<{unknown}, FxHasher>() -> HashSet<{unknown}, FxHasher>
            174..194 'FxHash...ault()': HashSet<{unknown}, FxHasher>
        "#]],
    );
}

#[test]
fn issue_2669() {
    check_infer(
        r#"
        trait A {}
        trait Write {}
        struct Response<T> {}

        trait D {
            fn foo();
        }

        impl<T:A> D for Response<T> {
            fn foo() {
                end();
                fn end<W: Write>() {
                    let _x: T =  loop {};
                }
            }
        }
        "#,
        expect![[r#"
            119..214 '{     ...     }': ()
            129..132 'end': fn end<{unknown}>()
            129..134 'end()': ()
            163..208 '{     ...     }': ()
            181..183 '_x': !
            190..197 'loop {}': !
            195..197 '{}': ()
        "#]],
    )
}

#[test]
fn issue_2705() {
    check_infer(
        r#"
        trait Trait {}
        fn test() {
            <Trait<u32>>::foo()
        }
        "#,
        expect![[r#"
            25..52 '{     ...oo() }': ()
            31..48 '<Trait...>::foo': {unknown}
            31..50 '<Trait...:foo()': ()
        "#]],
    );
}

#[test]
fn issue_2683_chars_impl() {
    check_types(
        r#"
//- minicore: iterator
pub struct Chars<'a> {}
impl<'a> Iterator for Chars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> { loop {} }
}

fn test() {
    let chars: Chars<'_>;
    (chars.next(), chars.nth(1));
} //^^^^^^^^^^^^^^^^^^^^^^^^^^^^ (Option<char>, Option<char>)
"#,
    );
}

#[test]
fn issue_3642_bad_macro_stackover() {
    check_no_mismatches(
        r#"
#[macro_export]
macro_rules! match_ast {
    (match $node:ident { $($tt:tt)* }) => { match_ast!(match ($node) { $($tt)* }) };

    (match ($node:expr) {
        $( ast::$ast:ident($it:ident) => $res:expr, )*
        _ => $catch_all:expr $(,)?
    }) => {{
        $( if let Some($it) = ast::$ast::cast($node.clone()) { $res } else )*
        { $catch_all }
    }};
}

fn main() {
    let anchor = match_ast! {
        match parent {
            as => {},
            _ => return None
        }
    };
}"#,
    );
}

#[test]
fn issue_3999_slice() {
    check_infer(
        r#"
        fn foo(params: &[usize]) {
            match params {
                [ps @ .., _] => {}
            }
        }
        "#,
        expect![[r#"
            7..13 'params': &[usize]
            25..80 '{     ...   } }': ()
            31..78 'match ...     }': ()
            37..43 'params': &[usize]
            54..66 '[ps @ .., _]': [usize]
            55..62 'ps @ ..': &[usize]
            60..62 '..': [usize]
            64..65 '_': usize
            70..72 '{}': ()
        "#]],
    );
}

#[test]
fn issue_3999_struct() {
    // rust-analyzer should not panic on seeing this malformed
    // record pattern.
    check_infer(
        r#"
        struct Bar {
            a: bool,
        }
        fn foo(b: Bar) {
            match b {
                Bar { a: .. } => {},
            }
        }
        "#,
        expect![[r#"
            35..36 'b': Bar
            43..95 '{     ...   } }': ()
            49..93 'match ...     }': ()
            55..56 'b': Bar
            67..80 'Bar { a: .. }': Bar
            76..78 '..': bool
            84..86 '{}': ()
        "#]],
    );
}

#[test]
fn issue_4235_name_conflicts() {
    check_infer(
        r#"
        struct FOO {}
        static FOO:FOO = FOO {};

        impl FOO {
            fn foo(&self) {}
        }

        fn main() {
            let a = &FOO;
            a.foo();
        }
        "#,
        expect![[r#"
            31..37 'FOO {}': FOO
            63..67 'self': &FOO
            69..71 '{}': ()
            85..119 '{     ...o(); }': ()
            95..96 'a': &FOO
            99..103 '&FOO': &FOO
            100..103 'FOO': FOO
            109..110 'a': &FOO
            109..116 'a.foo()': ()
        "#]],
    );
}

#[test]
fn issue_4465_dollar_crate_at_type() {
    check_infer(
        r#"
        pub struct Foo {}
        pub fn anything<T>() -> T {
            loop {}
        }
        macro_rules! foo {
            () => {{
                let r: $crate::Foo = anything();
                r
            }};
        }
        fn main() {
            let _a = foo!();
        }
        "#,
        expect![[r#"
            44..59 '{     loop {} }': T
            50..57 'loop {}': !
            55..57 '{}': ()
            !0..31 '{letr:...g();r}': Foo
            !4..5 'r': Foo
            !18..26 'anything': fn anything<Foo>() -> Foo
            !18..28 'anything()': Foo
            !29..30 'r': Foo
            163..187 '{     ...!(); }': ()
            173..175 '_a': Foo
        "#]],
    );
}

#[test]
fn issue_6811() {
    check_infer(
        r#"
        macro_rules! profile_function {
            () => {
                let _a = 1;
                let _b = 1;
            };
        }
        fn main() {
            profile_function!();
        }
        "#,
        expect![[r#"
            !3..5 '_a': i32
            !6..7 '1': i32
            !11..13 '_b': i32
            !14..15 '1': i32
            103..131 '{     ...!(); }': ()
        "#]],
    );
}

#[test]
fn issue_4053_diesel_where_clauses() {
    check_infer(
        r#"
        trait BoxedDsl<DB> {
            type Output;
            fn internal_into_boxed(self) -> Self::Output;
        }

        struct SelectStatement<From, Select, Distinct, Where, Order, LimitOffset, GroupBy, Locking> {
            order: Order,
        }

        trait QueryFragment<DB: Backend> {}

        trait Into<T> { fn into(self) -> T; }

        impl<F, S, D, W, O, LOf, DB> BoxedDsl<DB>
            for SelectStatement<F, S, D, W, O, LOf, G>
        where
            O: Into<dyn QueryFragment<DB>>,
        {
            type Output = XXX;

            fn internal_into_boxed(self) -> Self::Output {
                self.order.into();
            }
        }
        "#,
        expect![[r#"
            65..69 'self': Self
            267..271 'self': Self
            466..470 'self': SelectStatement<F, S, D, W, O, LOf, {unknown}, {unknown}>
            488..522 '{     ...     }': ()
            498..502 'self': SelectStatement<F, S, D, W, O, LOf, {unknown}, {unknown}>
            498..508 'self.order': O
            498..515 'self.o...into()': dyn QueryFragment<DB>
        "#]],
    );
}

#[test]
fn issue_4953() {
    check_infer(
        r#"
        pub struct Foo(pub i64);
        impl Foo {
            fn test() -> Self { Self(0i64) }
        }
        "#,
        expect![[r#"
            58..72 '{ Self(0i64) }': Foo
            60..64 'Self': Foo(i64) -> Foo
            60..70 'Self(0i64)': Foo
            65..69 '0i64': i64
        "#]],
    );
    check_infer(
        r#"
        pub struct Foo<T>(pub T);
        impl Foo<i64> {
            fn test() -> Self { Self(0i64) }
        }
        "#,
        expect![[r#"
            64..78 '{ Self(0i64) }': Foo<i64>
            66..70 'Self': Foo<i64>(i64) -> Foo<i64>
            66..76 'Self(0i64)': Foo<i64>
            71..75 '0i64': i64
        "#]],
    );
}

#[test]
fn issue_4931() {
    check_infer(
        r#"
        trait Div<T> {
            type Output;
        }

        trait CheckedDiv: Div<()> {}

        trait PrimInt: CheckedDiv<Output = ()> {
            fn pow(self);
        }

        fn check<T: PrimInt>(i: T) {
            i.pow();
        }
        "#,
        expect![[r#"
            117..121 'self': Self
            148..149 'i': T
            154..170 '{     ...w(); }': ()
            160..161 'i': T
            160..167 'i.pow()': ()
        "#]],
    );
}

#[test]
fn issue_4885() {
    check_infer(
        r#"
        //- minicore: coerce_unsized, future
        use core::future::Future;
        trait Foo<R> {
            type Bar;
        }
        fn foo<R, K>(key: &K) -> impl Future<Output = K::Bar>
        where
            K: Foo<R>,
        {
            bar(key)
        }
        fn bar<R, K>(key: &K) -> impl Future<Output = K::Bar>
        where
            K: Foo<R>,
        {
        }
        "#,
        expect![[r#"
            70..73 'key': &K
            132..148 '{     ...key) }': impl Future<Output = <K as Foo<R>>::Bar>
            138..141 'bar': fn bar<R, K>(&K) -> impl Future<Output = <K as Foo<R>>::Bar>
            138..146 'bar(key)': impl Future<Output = <K as Foo<R>>::Bar>
            142..145 'key': &K
            162..165 'key': &K
            224..227 '{ }': ()
        "#]],
    );
}

#[test]
fn issue_4800() {
    check_infer(
        r#"
        trait Debug {}

        struct Foo<T>;

        type E1<T> = (T, T, T);
        type E2<T> = E1<E1<E1<(T, T, T)>>>;

        impl Debug for Foo<E2<()>> {}

        struct Request;

        pub trait Future {
            type Output;
        }

        pub struct PeerSet<D>;

        impl<D> Service<Request> for PeerSet<D>
        where
            D: Discover,
            D::Key: Debug,
        {
            type Error = ();
            type Future = dyn Future<Output = Self::Error>;

            fn call(&mut self) -> Self::Future {
                loop {}
            }
        }

        pub trait Discover {
            type Key;
        }

        pub trait Service<Request> {
            type Error;
            type Future: Future<Output = Self::Error>;
            fn call(&mut self) -> Self::Future;
        }
        "#,
        expect![[r#"
            379..383 'self': &mut PeerSet<D>
            401..424 '{     ...     }': dyn Future<Output = ()>
            411..418 'loop {}': !
            416..418 '{}': ()
            575..579 'self': &mut Self
        "#]],
    );
}

#[test]
fn issue_4966() {
    check_infer(
        r#"
        //- minicore: deref
        pub trait IntoIterator {
            type Item;
        }

        struct Repeat<A> { element: A }

        struct Map<F> { f: F }

        struct Vec<T> {}

        impl<T> core::ops::Deref for Vec<T> {
            type Target = [T];
        }

        fn from_iter<A, T: IntoIterator<Item = A>>(iter: T) -> Vec<A> {}

        fn main() {
            let inner = Map { f: |_: &f64| 0.0 };

            let repeat = Repeat { element: inner };

            let vec = from_iter(repeat);

            vec.foo_bar();
        }
        "#,
        expect![[r#"
            225..229 'iter': T
            244..246 '{}': ()
            258..402 '{     ...r(); }': ()
            268..273 'inner': Map<|&f64| -> f64>
            276..300 'Map { ... 0.0 }': Map<|&f64| -> f64>
            285..298 '|_: &f64| 0.0': |&f64| -> f64
            286..287 '_': &f64
            295..298 '0.0': f64
            311..317 'repeat': Repeat<Map<|&f64| -> f64>>
            320..345 'Repeat...nner }': Repeat<Map<|&f64| -> f64>>
            338..343 'inner': Map<|&f64| -> f64>
            356..359 'vec': Vec<IntoIterator::Item<Repeat<Map<|&f64| -> f64>>>>
            362..371 'from_iter': fn from_iter<IntoIterator::Item<Repeat<Map<|&f64| -> f64>>>, Repeat<Map<|&f64| -> f64>>>(Repeat<Map<|&f64| -> f64>>) -> Vec<IntoIterator::Item<Repeat<Map<|&f64| -> f64>>>>
            362..379 'from_i...epeat)': Vec<IntoIterator::Item<Repeat<Map<|&f64| -> f64>>>>
            372..378 'repeat': Repeat<Map<|&f64| -> f64>>
            386..389 'vec': Vec<IntoIterator::Item<Repeat<Map<|&f64| -> f64>>>>
            386..399 'vec.foo_bar()': {unknown}
        "#]],
    );
}

#[test]
fn issue_6628() {
    check_infer(
        r#"
//- minicore: fn
struct S<T>();
impl<T> S<T> {
    fn f(&self, _t: T) {}
    fn g<F: FnOnce(&T)>(&self, _f: F) {}
}
fn main() {
    let s = S();
    s.g(|_x| {});
    s.f(10);
}
"#,
        expect![[r#"
            40..44 'self': &S<T>
            46..48 '_t': T
            53..55 '{}': ()
            81..85 'self': &S<T>
            87..89 '_f': F
            94..96 '{}': ()
            109..160 '{     ...10); }': ()
            119..120 's': S<i32>
            123..124 'S': S<i32>() -> S<i32>
            123..126 'S()': S<i32>
            132..133 's': S<i32>
            132..144 's.g(|_x| {})': ()
            136..143 '|_x| {}': |&i32| -> ()
            137..139 '_x': &i32
            141..143 '{}': ()
            150..151 's': S<i32>
            150..157 's.f(10)': ()
            154..156 '10': i32
        "#]],
    );
}

#[test]
fn issue_6852() {
    check_infer(
        r#"
//- minicore: deref
use core::ops::Deref;

struct BufWriter {}

struct Mutex<T> {}
struct MutexGuard<'a, T> {}
impl<T> Mutex<T> {
    fn lock(&self) -> MutexGuard<'_, T> {}
}
impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;
}
fn flush(&self) {
    let w: &Mutex<BufWriter>;
    *(w.lock());
}
"#,
        expect![[r#"
            123..127 'self': &Mutex<T>
            150..152 '{}': ()
            234..238 'self': &{unknown}
            240..290 '{     ...()); }': ()
            250..251 'w': &Mutex<BufWriter>
            276..287 '*(w.lock())': BufWriter
            278..279 'w': &Mutex<BufWriter>
            278..286 'w.lock()': MutexGuard<BufWriter>
        "#]],
    );
}

#[test]
fn param_overrides_fn() {
    check_types(
        r#"
        fn example(example: i32) {
            fn f() {}
            example;
          //^^^^^^^ i32
        }
        "#,
    )
}

#[test]
fn lifetime_from_chalk_during_deref() {
    check_types(
        r#"
//- minicore: deref
struct Box<T: ?Sized> {}
impl<T: ?Sized> core::ops::Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        loop {}
    }
}

trait Iterator {
    type Item;
}

pub struct Iter<'a, T: 'a> {
    inner: Box<dyn IterTrait<'a, T, Item = &'a T> + 'a>,
}

trait IterTrait<'a, T: 'a>: Iterator<Item = &'a T> {
    fn clone_box(&self);
}

fn clone_iter<T>(s: Iter<T>) {
    s.inner.clone_box();
  //^^^^^^^^^^^^^^^^^^^ ()
}
"#,
    )
}

#[test]
fn issue_8686() {
    check_infer(
        r#"
pub trait Try: FromResidual {
    type Output;
    type Residual;
}
pub trait FromResidual<R = <Self as Try>::Residual> {
     fn from_residual(residual: R) -> Self;
}

struct ControlFlow<B, C>;
impl<B, C> Try for ControlFlow<B, C> {
    type Output = C;
    type Residual = ControlFlow<B, !>;
}
impl<B, C> FromResidual for ControlFlow<B, C> {
    fn from_residual(r: ControlFlow<B, !>) -> Self { ControlFlow }
}

fn test() {
    ControlFlow::from_residual(ControlFlow::<u32, !>);
}
        "#,
        expect![[r#"
            144..152 'residual': R
            365..366 'r': ControlFlow<B, !>
            395..410 '{ ControlFlow }': ControlFlow<B, C>
            397..408 'ControlFlow': ControlFlow<B, C>
            424..482 '{     ...!>); }': ()
            430..456 'Contro...sidual': fn from_residual<ControlFlow<u32, {unknown}>, ControlFlow<u32, !>>(ControlFlow<u32, !>) -> ControlFlow<u32, {unknown}>
            430..479 'Contro...2, !>)': ControlFlow<u32, {unknown}>
            457..478 'Contro...32, !>': ControlFlow<u32, !>
        "#]],
    );
}

#[test]
fn cfg_tail() {
    // https://github.com/rust-analyzer/rust-analyzer/issues/8378
    check_infer(
        r#"
        fn fake_tail(){
            { "first" }
            #[cfg(never)] 9
        }
        fn multiple_fake(){
            { "fake" }
            { "fake" }
            { "second" }
            #[cfg(never)] { 11 }
            #[cfg(never)] 12;
            #[cfg(never)] 13
        }
        fn no_normal_tail(){
            { "third" }
            #[cfg(never)] 14;
            #[cfg(never)] 15;
        }
        fn no_actual_tail(){
            { "fourth" };
            #[cfg(never)] 14;
            #[cfg(never)] 15
        }
        "#,
        expect![[r#"
            14..53 '{     ...)] 9 }': &str
            20..31 '{ "first" }': &str
            22..29 '"first"': &str
            72..190 '{     ...] 13 }': &str
            78..88 '{ "fake" }': &str
            80..86 '"fake"': &str
            93..103 '{ "fake" }': &str
            95..101 '"fake"': &str
            108..120 '{ "second" }': &str
            110..118 '"second"': &str
            210..273 '{     ... 15; }': &str
            216..227 '{ "third" }': &str
            218..225 '"third"': &str
            293..357 '{     ...] 15 }': ()
            299..311 '{ "fourth" }': &str
            301..309 '"fourth"': &str
        "#]],
    )
}

#[test]
fn impl_trait_in_option_9530() {
    check_types(
        r#"
//- minicore: sized
struct Option<T>;
impl<T> Option<T> {
    fn unwrap(self) -> T { loop {} }
}
fn make() -> Option<impl Copy> { Option }
trait Copy {}
fn test() {
    let o = make();
    o.unwrap();
  //^^^^^^^^^^ impl Copy
}
        "#,
    )
}

#[test]
fn bare_dyn_trait_binders_9639() {
    check_no_mismatches(
        r#"
//- minicore: fn, coerce_unsized
fn infix_parse<T, S>(_state: S, _level_code: &Fn(S)) -> T {
    loop {}
}

fn parse_arule() {
    infix_parse((), &(|_recurse| ()))
}
        "#,
    )
}

#[test]
fn call_expected_type_closure() {
    check_types(
        r#"
//- minicore: fn, option

fn map<T, U>(o: Option<T>, f: impl FnOnce(T) -> U) -> Option<U> { loop {} }
struct S {
    field: u32
}

fn test() {
    let o = Some(S { field: 2 });
    let _: Option<()> = map(o, |s| { s.field; });
                                  // ^^^^^^^ u32
}
        "#,
    );
}

#[test]
fn coerce_diesel_panic() {
    check_no_mismatches(
        r#"
//- minicore: option

trait TypeMetadata {
    type MetadataLookup;
}

pub struct Output<'a, T, DB>
where
    DB: TypeMetadata,
    DB::MetadataLookup: 'a,
{
    out: T,
    metadata_lookup: Option<&'a DB::MetadataLookup>,
}

impl<'a, T, DB: TypeMetadata> Output<'a, T, DB> {
    pub fn new(out: T, metadata_lookup: &'a DB::MetadataLookup) -> Self {
        Output {
            out,
            metadata_lookup: Some(metadata_lookup),
        }
    }
}
        "#,
    );
}

#[test]
fn bitslice_panic() {
    check_no_mismatches(
        r#"
//- minicore: option, deref

pub trait BitView {
    type Store;
}

pub struct Lsb0;

pub struct BitArray<V: BitView> { }

pub struct BitSlice<T> { }

impl<V: BitView> core::ops::Deref for BitArray<V> {
    type Target = BitSlice<V::Store>;
}

impl<T> BitSlice<T> {
    pub fn split_first(&self) -> Option<(T, &Self)> { loop {} }
}

fn multiexp_inner() {
    let exp: &BitArray<Foo>;
    exp.split_first();
}
        "#,
    );
}

#[test]
fn macro_expands_to_impl_trait() {
    check_no_mismatches(
        r#"
trait Foo {}

macro_rules! ty {
    () => {
        impl Foo
    }
}

fn foo(_: ty!()) {}

fn bar() {
    foo(());
}
    "#,
    )
}
