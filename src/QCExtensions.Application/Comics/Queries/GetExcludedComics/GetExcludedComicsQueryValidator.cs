using FluentValidation;
using QCExtensions.Application.Comics.Models;

namespace QCExtensions.Application.Comics.Queries.GetExcludedComics
{
	public class GetExcludedComicsQueryValidator : AbstractValidator<GetExcludedComicsQuery>
	{
		public GetExcludedComicsQueryValidator()
		{
			RuleFor(x => x.ExclusionType).NotEqual(Exclusion.None);
		}
	}
}
