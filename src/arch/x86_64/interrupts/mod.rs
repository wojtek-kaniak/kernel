use crate::common::mem::Bittable;

use self::idt::IdtVector;

pub mod idt;

pub trait Interrupt {
    type Handler;
    const VECTOR: IdtVector;
}

macro_rules! define_interrupt {
    ($name:ident = $vector:expr, $handler:ty) => {
        pub struct $name {}

        impl Interrupt for $name {
            type Handler = $handler;
            const VECTOR: IdtVector = $vector;
        }
    };
}

pub trait InterruptHandler {
    type Interrupt: self::Interrupt;

    /// This function should only be called by hardware
    #[deprecated = "should not be called directly"]
    extern "C" fn invoke() -> !;
}

macro_rules! _define_interrupt_handler_asm {
    (($arg:ident : $argtype:ty)) => {
        {
            ::static_assertions::const_assert_eq!(
                ::core::mem::size_of::<<$argtype as ::core::ops::Deref>::Target>(),
                ::core::mem::size_of::<StackFrame>()
            );
            ::static_assertions::const_assert_eq!(
                ::core::mem::align_of::<<$argtype as ::core::ops::Deref>::Target>(),
                ::core::mem::align_of::<StackFrame>()
            );

            ::core::arch::asm!(
                "
                push    r11
                push    r10
                push    r9
                push    r8
                push    rdi
                push    rsi
                push    rdx
                push    rcx
                push    rax
                cld
                lea     rdi, [rsp + 72]
                call    {}
                pop     rax
                pop     rcx
                pop     rdx
                pop     rsi
                pop     rdi
                pop     r8
                pop     r9
                pop     r10
                pop     r11
                iretq
                ",
                sym Self::handler,
                options(noreturn)
            )
        }
    };
    (($arg1:ident : $argtype1:ty , $arg2:ident : $argtype2:ty)) => {
        {
            ::static_assertions::const_assert_eq!(
                ::core::mem::size_of::<<$argtype1 as ::core::ops::Deref>::Target>(),
                ::core::mem::size_of::<StackFrame>()
            );
            ::static_assertions::const_assert_eq!(
                ::core::mem::align_of::<<$argtype1 as ::core::ops::Deref>::Target>(),
                ::core::mem::align_of::<StackFrame>()
            );
            ::static_assertions::assert_impl_all!(
                <$argtype1 as ::core::ops::Deref>::Target: $crate::common::mem::Bittable
            );
            ::static_assertions::const_assert_eq!(::core::mem::size_of::<$argtype2>(), ::core::mem::size_of::<ErrorCode>());
            ::static_assertions::const_assert_eq!(::core::mem::align_of::<$argtype2>(), ::core::mem::align_of::<ErrorCode>());
            ::static_assertions::assert_impl_all!($argtype2: $crate::common::mem::Bittable);

            ::core::arch::asm!(
                "
                push    rax
                push    r11
                push    r10
                push    r9
                push    r8
                push    rdi
                push    rsi
                push    rdx
                push    rcx
                push    rax
                push    rax
                cld
                mov     rsi, qword ptr [rsp + 88]
                lea     rdi, [rsp + 96]
                call    {}
                add     rsp, 8
                pop     rax
                pop     rcx
                pop     rdx
                pop     rsi
                pop     rdi
                pop     r8
                pop     r9
                pop     r10
                pop     r11
                add     rsp, 16
                iretq
                ",
                sym Self::handler,
                options(noreturn)
            )
        }
    };
}
#[doc(hidden)]
use _define_interrupt_handler_asm;

macro_rules! define_interrupt_handler {
    {handler $name:ident $args:tt for $interrupt:ty $body:block } => {
        pub enum $name {}

        impl $name {
            // Force `handler` to have the correct signature
            const _HANDLER: <$interrupt as $crate::arch::x86_64::interrupts::Interrupt>::Handler = Self::handler;

            extern "sysv64" fn handler $args -> () $body
        }

        impl InterruptHandler for $name {
            type Interrupt = $interrupt;

            #[naked]
            extern "C" fn invoke() -> ! {
                unsafe {
                    $crate::arch::x86_64::interrupts::_define_interrupt_handler_asm!($args)
                }
            }
        }
    };
    {handler $name:ident $args:tt for $interrupt:ty $body:block $($tail:tt)*} => {
        define_interrupt_handler!{
            handler $name $args for $interrupt $body
        }
        define_interrupt_handler! {
            $($tail)*
        }
    };
}
pub(crate) use define_interrupt_handler;

// TODO: store the stack frame
pub struct StackFrame;

unsafe impl Bittable for StackFrame {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ErrorCode(pub usize);

unsafe impl Bittable for ErrorCode {}

impl From<usize> for ErrorCode {
    fn from(value: usize) -> Self {
        ErrorCode(value)
    }
}

impl From<ErrorCode> for usize {
    fn from(val: ErrorCode) -> Self {
        val.0
    }
}

type InterruptHandlerType = extern "sysv64" fn(&StackFrame);
type InterruptWithErrorCodeHandlerType = extern "sysv64" fn(&StackFrame, ErrorCode);

define_interrupt!(IntegerDivideByZero = IdtVector::INTEGER_DIVIDE_BY_ZERO, InterruptHandlerType);
define_interrupt!(Debug = IdtVector::DEBUG, InterruptHandlerType);
define_interrupt!(NonMaskableInterrupt = IdtVector::NON_MASKABLE_INTERRUPT, InterruptHandlerType);
define_interrupt!(Breakpoint = IdtVector::BREAKPOINT, InterruptHandlerType);
define_interrupt!(Overflow = IdtVector::OVERFLOW, InterruptHandlerType);
define_interrupt!(BoundRangeExceeded = IdtVector::BOUND_RANGE_EXCEEDED, InterruptHandlerType);
define_interrupt!(InvalidOpcode = IdtVector::INVALID_OPCODE, InterruptHandlerType);
define_interrupt!(DeviceNotAvailable = IdtVector::DEVICE_NOT_AVAILABLE, InterruptHandlerType);
define_interrupt!(DoubleFault = IdtVector::DOUBLE_FAULT, InterruptWithErrorCodeHandlerType);
define_interrupt!(CoprocessorSegmentOverrun = IdtVector::COPROCESSOR_SEGMENT_OVERRUN, InterruptHandlerType);
define_interrupt!(InvalidTTS = IdtVector::INVALID_TTS, InterruptWithErrorCodeHandlerType);
define_interrupt!(SegmentNotPresent = IdtVector::SEGMENT_NOT_PRESENT, InterruptWithErrorCodeHandlerType);
define_interrupt!(StackSegmentFault = IdtVector::STACK_SEGMENT_FAULT, InterruptWithErrorCodeHandlerType);
define_interrupt!(GeneralProtection = IdtVector::GENERAL_PROTECTION, InterruptWithErrorCodeHandlerType);
define_interrupt!(PageFault = IdtVector::PAGE_FAULT, InterruptWithErrorCodeHandlerType);
define_interrupt!(X87FloatingPointError = IdtVector::X87_FLOATING_POINT_ERROR, InterruptHandlerType);
define_interrupt!(AlignmentCheck = IdtVector::ALIGNMENT_CHECK, InterruptWithErrorCodeHandlerType);
define_interrupt!(MachineCheck = IdtVector::MACHINE_CHECK, InterruptHandlerType);
define_interrupt!(SimdFloatingPointException = IdtVector::SIMD_FLOATING_POINT_EXCEPTION, InterruptHandlerType);
define_interrupt!(VirtualizationException = IdtVector::VIRTUALIZATION_EXCEPTION, InterruptHandlerType);
define_interrupt!(ControlProtectionException = IdtVector::CONTROL_PROTECTION_EXCEPTION, InterruptWithErrorCodeHandlerType);
define_interrupt!(HypervisorInjectionException = IdtVector::HYPERVISOR_INJECTION_EXCEPTION, InterruptHandlerType);
define_interrupt!(VmmCommunicationException = IdtVector::VMM_COMMUNICATION_EXCEPTION, InterruptWithErrorCodeHandlerType);
define_interrupt!(SecurityException = IdtVector::SECURITY_EXCEPTION, InterruptWithErrorCodeHandlerType);
