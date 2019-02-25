using MediatR;
using QCExtensions.Application.Items.Models;

namespace QCExtensions.Application.Items.Queries.GetItem
{
	public class GetItemQuery : IRequest<ItemDto>
	{
		public int ItemId { get; set; }
	}
}
