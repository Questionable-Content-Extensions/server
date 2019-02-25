using MediatR;
using QCExtensions.Application.Items.Models;
using System.Collections.Generic;

namespace QCExtensions.Application.Items.Queries.GetItemImages
{
	public class GetItemImagesQuery : IRequest<List<ItemImageDto>>
	{
		public int ItemId { get; set; }
	}
}
