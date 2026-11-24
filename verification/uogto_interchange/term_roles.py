"""Witness roles for properties whose pinned definitions omit a domain or range."""

from rdflib import Namespace, OWL, XSD

CORE = Namespace("https://w3id.org/uogto/core#")
EXTENSIONS = Namespace("https://w3id.org/uogto/extensions#")
HTA = Namespace("https://w3id.org/uogto/packs/hta#")

# Each tuple gives allowed witness types for a missing domain and range.
# Empty tuples defer to a declared constraint. These roles qualify the
# examples used as evidence for the prose definitions, within their context.
PROSE_ROLES = {
    CORE.assignsPayoff: ((CORE.Outcome,), (CORE.Payoff, CORE.PayoffProfile)),
    CORE.belongsToPlayer: ((CORE.Action, EXTENSIONS.ActionSpace), ()),
    CORE.compiledToExecutionModel: ((CORE.GameSpecification,), (CORE.ExecutionModel,)),
    CORE.confidence: ((CORE.Belief,), ()),
    CORE.description: ((CORE.GameEntity,), ()),
    CORE.emitsEventTrace: ((CORE.PlaySession,), (CORE.EventTrace,)),
    CORE.governedByNorm: ((CORE.Institution,), (CORE.Norm,)),
    CORE.governedByRule: ((CORE.GameSpecification, CORE.GameInstance), (CORE.Rule,)),
    CORE.hasAction: ((EXTENSIONS.ActionSpace, CORE.ActionProfile), ()),
    CORE.hasActionProfile: ((CORE.GameEntity,), ()),
    CORE.hasActionSpace: ((CORE.GameSpecification,), (EXTENSIONS.ActionSpace,)),
    CORE.hasAvailableAction: ((CORE.State, CORE.GameNode), (CORE.Action,)),
    CORE.hasBelief: ((CORE.InformationSet,), (CORE.Belief,)),
    CORE.hasEquilibrium: ((CORE.GameSpecification,), (CORE.Equilibrium,)),
    CORE.hasInformationSet: ((CORE.State, CORE.GameNode), (CORE.InformationSet,)),
    CORE.hasInitialState: ((CORE.GameSpecification,), (CORE.State, CORE.GameNode)),
    CORE.hasPayoffMapping: ((CORE.GameSpecification,), (CORE.GameEntity,)),
    CORE.hasPlayer: ((CORE.GameSpecification, CORE.GameInstance), (CORE.Player,)),
    CORE.hasRepresentation: ((CORE.GameSpecification,), (CORE.GameRepresentation,)),
    CORE.hasRole: ((CORE.Player,), (CORE.Role,)),
    CORE.hasSolutionConcept: ((CORE.GameSpecification, CORE.GameSolution, EXTENSIONS.ComputationalProblem), ()),
    CORE.hasState: ((CORE.GameSpecification, CORE.GameInstance), (CORE.State, CORE.GameNode)),
    CORE.hasStrategy: ((CORE.Player,), (CORE.Strategy,)),
    CORE.hasStrategyProfile: ((CORE.Equilibrium,), (CORE.StrategyProfile,)),
    CORE.hasTerminalState: ((CORE.GameSpecification, CORE.GameInstance), (CORE.TerminalState,)),
    CORE.hasTransition: ((CORE.State, CORE.GameNode, CORE.Action), (CORE.Transition,)),
    CORE.hasUtilityFunction: ((CORE.Role,), (CORE.UtilityFunction,)),
    CORE.identifier: ((CORE.GameEntity,), ()),
    CORE.name: ((CORE.GameEntity,), ()),
    CORE.observes: ((CORE.Player, CORE.Agent), (CORE.Signal, CORE.State)),
    CORE.participatesIn: ((CORE.Player,), (CORE.PlaySession, CORE.GameSpecification)),
    CORE.payoffValue: ((CORE.Payoff, CORE.PlayerPayoffLink), ()),
    CORE.probability: ((CORE.Transition, CORE.Signal), (XSD.double, XSD.decimal, XSD.float, XSD.integer)),
    CORE.producesOutcome: ((CORE.Action, CORE.State, CORE.GameNode), (CORE.Outcome,)),
    CORE.selectsAction: ((CORE.Player, CORE.Agent), (CORE.Action,)),
    CORE.solvedBy: ((CORE.Equilibrium,), (CORE.Solver,)),
    CORE.sourceReference: ((CORE.GameEntity, OWL.Ontology), ()),
    CORE.timeIndex: ((CORE.State, CORE.GameNode, CORE.EventTrace), ()),
    CORE.timestamp: ((CORE.EventTrace,), ()),
    CORE.transitionsFrom: ((CORE.Transition,), (CORE.State, CORE.GameNode)),
    CORE.transitionsTo: ((CORE.Transition,), (CORE.State, CORE.GameNode)),
    CORE.utilityValue: ((CORE.Outcome,), ()),
    CORE.version: ((OWL.Ontology,), ()),
    HTA.thresholdValue: ((HTA.AppraisalThreshold,), ()),
}
