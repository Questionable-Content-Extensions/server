using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.AddItemToComic
{
	public class AddItemToComicCommandValidator : AbstractValidator<AddItemToComicCommand>
	{
		public AddItemToComicCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
			
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1).Unless(x => x.ItemId == AddItemToComicCommand.CreateNewItemId);
			RuleFor(x => x.ItemId).Equal(AddItemToComicCommand.CreateNewItemId).Unless(x => x.ItemId > 0);

			RuleFor(x => x.NewItemName).Empty().Unless(x => x.ItemId == AddItemToComicCommand.CreateNewItemId);
			RuleFor(x => x.NewItemType).Empty().Unless(x => x.ItemId == AddItemToComicCommand.CreateNewItemId);

			RuleFor(x => x.NewItemName).NotEmpty().Unless(x => x.ItemId != AddItemToComicCommand.CreateNewItemId);
			RuleFor(x => x.NewItemType).NotEmpty().Unless(x => x.ItemId != AddItemToComicCommand.CreateNewItemId);
		}
	}
}
