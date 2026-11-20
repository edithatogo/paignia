// OpenSpiel reference driver for test evidence. See docs/validation/cfr.md.
#include <iomanip>
#include <iostream>
#include <string_view>

#include "open_spiel/algorithms/cfr.h"
#include "open_spiel/algorithms/expected_returns.h"
#include "open_spiel/algorithms/tabular_exploitability.h"
#include "open_spiel/spiel.h"

int main() {
  const auto game = open_spiel::LoadGame("leduc_poker");
  if (game->NumPlayers() != 2) return 1;
  std::cout << "variant,iterations,first_player_value,second_player_value,nash_conv\n";
  std::cout << std::setprecision(17);
  struct Schedule {
    std::string_view name;
    bool alternating;
    bool linear_average;
    bool plus;
  };
  for (const auto schedule : {Schedule{"cfr_simultaneous", false, false, false},
                              Schedule{"cfr_plus_uniform_reverse", true, false, true},
                              Schedule{"cfr_plus_linear_reverse", true, true, true}}) {
    const auto variant = schedule.name;
    open_spiel::algorithms::CFRSolverBase solver(
        *game, schedule.alternating, schedule.linear_average, schedule.plus);
    for (int iteration = 1; iteration <= 4000; ++iteration) {
      solver.EvaluateAndUpdatePolicy();
      if (iteration != 1000 && iteration != 4000) continue;
      auto policy = solver.AveragePolicy();
      auto returns = open_spiel::algorithms::ExpectedReturns(
          *game->NewInitialState(), *policy, -1);
      const double nash_conv = open_spiel::algorithms::NashConv(*game, *policy);
      std::cout << variant << ',' << iteration << ',' << returns.at(0) << ','
                << returns.at(1) << ',' << nash_conv << '\n';
    }
  }
}
