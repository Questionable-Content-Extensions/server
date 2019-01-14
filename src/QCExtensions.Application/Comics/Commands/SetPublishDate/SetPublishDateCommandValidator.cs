using System;
using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.SetPublishDate
{
	public class SetPublishDateCommandValidator : AbstractValidator<SetPublishDateCommand>
	{
		public SetPublishDateCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
			RuleFor(x => x.PublishDate)
				.GreaterThan(new DateTime(2003, 8, 1))
				.LessThan(DateTime.UtcNow.AddMonths(1));
		}
	}
}
