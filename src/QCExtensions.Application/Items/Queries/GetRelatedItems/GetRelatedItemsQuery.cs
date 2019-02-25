using MediatR;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Enumerations;
using System.Collections.Generic;

namespace QCExtensions.Application.Items.Queries.GetRelatedItems
{
	public class GetRelatedItemsQuery : IRequest<List<ItemListDto>>
	{
		public int ItemId { get; set; }
		public ItemType Type { get; set; }
		public int Amount { get; set; } = 5;
	}
}
