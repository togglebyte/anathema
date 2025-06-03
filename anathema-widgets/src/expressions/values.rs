use anathema_state::CommonVal;
use anathema_strings::{HStrings, StrIndex};
use anathema_templates::expressions::Equality;

pub enum ValueKind {
    Common(CommonVal),
    String(StrIndex),
    Null,
}

impl ValueKind {
    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueKind::Common(_), ValueKind::Common(_)) => true,
            (ValueKind::String(_), ValueKind::String(_)) => true,
            (ValueKind::Null, ValueKind::Null) => true,
            _ => false,
        }
    }

    fn to_float(&self) -> Option<f64> {
        match self {
            ValueKind::Common(CommonVal::Int(i)) => Some(*i as f64),
            ValueKind::Common(CommonVal::Float(f)) => Some(*f),
            _ => None,
        }
    }

    fn equality(&self, other: &Self, strings: &HStrings<'_>, eq: Equality) -> bool {
        if !self.same(other) {
            return false;
        }

        panic!("implement this, it's a hot mess");

        // match eq {
        //     eq @ (Equality::Eq | Equality::NotEq) => {
        //         match (self, other) {
        //             (ValueKind::Common(lhs), ValueKind::Common(rhs)) => {
        //                 match (lhs, rhs) {
        //                     (CommonVal::Int(lhs), CommonVal::Float(rhs)) => todo!(),
        //                     (CommonVal::Float(lhs), CommonVal::Int(rhs)) => todo!(),
        //                     (lhs, rhs) => match eq {
        //                         Equality::Eq => lhs == rhs,
        //                         Equality::NotEq => lhs != rhs,
        //                         _ => unreachable!()
        //                     }
        //                 }
        //             },
        //             (ValueKind::String(lhs), ValueKind::String(rhs)) => {
        //                 let lhs = strings.get(*lhs);
        //                 let rhs = strings.get(*rhs);
        //                 lhs == rhs
        //             }
        //             _ => false,
        //         }
        //     }

        //     Equality::And => {
        //             (CommonVal::Bool(lhs), CommonVal::Bool(rhs)) => lhs && rhs,
        //     }
        //     Equality::Or => {
        //             (CommonVal::Bool(lhs), CommonVal::Bool(rhs)) => lhs || rhs,
        //     }

        //     eq @ (Equality::Gt | Equality::Gte | Equality::Lt | Equality::Lte) => {
        //         // TODO: this might get a bit cursed as all numbers are cast to float
        //         let Some(lhs) = self.to_float() else { return false };
        //         let Some(rhs) = other.to_float() else { return false };

        //         match (lhs, rhs) {
        //             (lhs, rhs) => match eq {
        //                 Equality::Gt => lhs.gt(&rhs),
        //                 Equality::Gte => lhs.ge(&rhs),
        //                 Equality::Lt => lhs.lt(&rhs),
        //                 Equality::Lte => lhs.le(&rhs),
        //                 _ => false,
        //             },
        //             _ => false,
        //         }
        //     }
        // }
    }
}
