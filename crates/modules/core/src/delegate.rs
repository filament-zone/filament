use sov_modules_api::Spec;

pub type Delegate<S> = <S as Spec>::Address;
pub type Eviction<S> = <S as Spec>::Address;
