use paignia_core::Bimatrix2;
use paignia_solve::InspectionRatio;

pub fn r(numerator: i64, denominator: i64) -> InspectionRatio {
    InspectionRatio::new(numerator, denominator).unwrap()
}

pub fn dominance(row: i64, column: i64) -> Bimatrix2<InspectionRatio> {
    Bimatrix2 {
        row_payoffs: [[r(row, 1); 2], [r(0, 1); 2]],
        column_payoffs: [[r(column, 1), r(0, 1)]; 2],
    }
}

pub fn pennies(sign: i64) -> Bimatrix2<InspectionRatio> {
    let high = r(sign, 1);
    let low = r(-sign, 1);
    Bimatrix2 {
        row_payoffs: [[high, low], [low, high]],
        column_payoffs: [[low, high], [high, low]],
    }
}
