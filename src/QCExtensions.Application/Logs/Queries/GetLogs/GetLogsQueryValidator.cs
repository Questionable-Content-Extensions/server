using FluentValidation;

namespace QCExtensions.Application.Logs.Queries.GetLogs
{
	public class GetLogsQueryValidator : AbstractValidator<GetLogsQuery>
	{
		public GetLogsQueryValidator()
		{
			RuleFor(x => x.Page).GreaterThanOrEqualTo(1);
		}
	}
}
