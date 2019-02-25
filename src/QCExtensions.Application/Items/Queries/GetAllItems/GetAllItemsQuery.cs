using MediatR;
using QCExtensions.Application.Items.Models;
using System.Collections.Generic;

namespace QCExtensions.Application.Items.Queries.GetAllItems
{
	public class GetAllItemsQuery : IRequest<List<ItemListDto>>
	{
	}
}
